use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Result};
use axum::http::StatusCode;
use config::CONFIG;
use futures::StreamExt;
use multimint::{
    fedimint_client::{oplog::UpdateStreamOrOutcome, ClientHandleArc},
    fedimint_core::{config::FederationId, core::OperationId, secp256k1::PublicKey, Amount},
    fedimint_ln_client::{LightningClientModule, LnReceiveState},
    fedimint_ln_common::lightning_invoice::{Bolt11Invoice, Bolt11InvoiceDescription, Description},
    MultiMint,
};
use nostr_sdk::{bitcoin::XOnlyPublicKey, secp256k1::Parity};
use tokio::task::spawn;
use tracing::{error, info};

use crate::{
    config,
    error::AppError,
    model::{
        invoices::{Invoice, InvoiceForCreate, InvoiceState},
        users::User,
        Db,
    },
    nostr::Nostr,
    router::handlers::lnurlp::callback::LnurlCallbackParams,
};

#[derive(Clone)]
pub struct AppState {
    pub mm: MultiMint,
    pub db: Db,
    pub nostr: Nostr,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let mm = MultiMint::new(CONFIG.fm_db_path.clone()).await?;
        let db = Db::new(CONFIG.pg_db.clone()).await?;
        let nostr = Nostr::new(&CONFIG.nostr_nsec)?;

        nostr.add_relays(&CONFIG.nostr_relays).await?;
        db.setup_schema().await?;

        Ok(Self { mm, db, nostr })
    }

    pub async fn register_federations(&mut self) -> Result<()> {
        for invite_code in &CONFIG.federation_invite_codes {
            self.mm.register_new(invite_code.clone(), None).await?;
        }
        Ok(())
    }

    pub async fn handle_pending_invoices(&self) -> Result<()> {
        let pending_invoices = self
            .db
            .invoice()
            .get_by_state(InvoiceState::Pending)
            .await?;
        let pending_invoices_by_federation = self.group_invoices_by_federation(pending_invoices);

        for (federation_id, invoices) in pending_invoices_by_federation {
            self.handle_federation_invoices(&federation_id, invoices)
                .await?;
        }

        Ok(())
    }

    fn group_invoices_by_federation(
        &self,
        invoices: Vec<Invoice>,
    ) -> HashMap<String, Vec<Invoice>> {
        invoices
            .into_iter()
            .fold(HashMap::new(), |mut acc, invoice| {
                acc.entry(invoice.federation_id.clone())
                    .or_default()
                    .push(invoice);
                acc
            })
    }

    async fn handle_federation_invoices(
        &self,
        federation_id: &str,
        invoices: Vec<Invoice>,
    ) -> Result<()> {
        info!(
            "Processing {} invoices for federation: {}",
            invoices.len(),
            federation_id
        );

        let federation_id =
            FederationId::from_str(federation_id).context("Invalid federation ID")?;

        let client = self
            .mm
            .clients
            .lock()
            .await
            .get(&federation_id)
            .cloned()
            .context("Client not found")?;

        let ln = client.get_first_module::<LightningClientModule>();

        for invoice in invoices {
            self.subscribe_to_invoice(&ln, invoice).await?;
        }

        Ok(())
    }

    pub async fn subscribe_to_invoice(
        &self,
        ln: &LightningClientModule,
        invoice: Invoice,
    ) -> Result<()> {
        let op_id = invoice.op_id.parse().context("Invalid op_id")?;
        let subscription = ln
            .subscribe_ln_receive(op_id)
            .await
            .context("Failed to subscribe to invoice")?;

        self.spawn_invoice_subscription(invoice, subscription).await
    }

    async fn spawn_invoice_subscription(
        &self,
        invoice: Invoice,
        subscription: UpdateStreamOrOutcome<LnReceiveState>,
    ) -> Result<()> {
        let invoice_db = self.db.invoice().clone();
        let nostr = self.nostr.clone();

        spawn(async move {
            info!("Monitoring invoice: {}", invoice.op_id);
            let mut stream = subscription.into_stream();
            while let Some(op_state) = stream.next().await {
                match op_state {
                    LnReceiveState::Canceled { reason } => {
                        error!("Invoice {} canceled: {:?}", invoice.op_id, reason);
                        if let Err(e) = invoice_db
                            .update_state(invoice.id, InvoiceState::Cancelled)
                            .await
                        {
                            error!("Failed to update invoice state: {}", e);
                        }
                    }
                    LnReceiveState::Claimed => {
                        info!("Invoice {} claimed", invoice.op_id);
                        if let Err(e) = invoice_db
                            .update_state(invoice.id, InvoiceState::Settled)
                            .await
                        {
                            error!("Failed to update invoice state: {}", e);
                        }
                        if let Err(e) = nostr.notify_user_invoice_settled(invoice).await {
                            error!("Failed to notify user of settled invoice: {}", e);
                        }
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    async fn create_invoice_for_user_tweaked(
        ln: &LightningClientModule,
        params: &LnurlCallbackParams,
        user: &User,
        tweak: i64,
    ) -> Result<(OperationId, Bolt11Invoice, [u8; 32])> {
        let xonly_pubkey = XOnlyPublicKey::from_str(&user.pubkey)?;
        let pubkey = PublicKey::from_str(&xonly_pubkey.public_key(Parity::Even).to_string())?;
        ln.create_bolt11_invoice_for_user_tweaked(
            Amount {
                msats: params.amount,
            },
            Bolt11InvoiceDescription::Direct(&Description::new(
                params
                    .comment
                    .clone()
                    .unwrap_or_else(|| "hermes address payment".to_string()),
            )?),
            None,
            pubkey,
            tweak as u64,
            (),
            None,
        )
        .await
    }

    pub async fn create_invoice_store_and_notify(
        &self,
        ln: &LightningClientModule,
        user: &User,
        params: &LnurlCallbackParams,
        federation_id: FederationId,
    ) -> Result<(OperationId, Invoice)> {
        let mut tweak = user.last_tweak + 1;
        let (op_id, invoice, _) = loop {
            match Self::create_invoice_for_user_tweaked(ln, params, user, tweak).await {
                Ok(result) => break result,
                Err(e) if e.to_string().contains("already exists") => {
                    info!("Invoice already exists, trying next tweak");
                    tweak += 1;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        };
        self.db.users().update_tweak(user, tweak).await?;

        let stored_invoice = self
            .db
            .invoice()
            .create(InvoiceForCreate {
                op_id: op_id.fmt_full().to_string(),
                federation_id: federation_id.to_string(),
                user_id: user.id,
                user_pubkey: user.pubkey.clone(),
                amount: params.amount as i64,
                bolt11: invoice.to_string(),
                tweak,
                state: InvoiceState::Pending,
            })
            .await?;

        self.subscribe_to_invoice(ln, stored_invoice.clone())
            .await?;

        Ok((op_id, stored_invoice))
    }

    pub async fn get_federation_and_client(
        &self,
        user: &User,
    ) -> Result<(FederationId, ClientHandleArc), AppError> {
        info!("Getting federation and client for user: {}", user.name);

        let federation_id = FederationId::from_str(&user.federation_ids[0]).map_err(|e| {
            let error_msg = format!("Invalid federation_id for user {}: {}", user.name, e);
            tracing::error!("{}", error_msg);
            AppError::new(StatusCode::BAD_REQUEST, anyhow::anyhow!(error_msg))
        })?;

        info!("Federation ID parsed: {:?}", federation_id);

        let locked_clients = self.mm.clients.lock().await;
        let client = locked_clients.get(&federation_id).cloned().ok_or_else(|| {
            let error_msg = format!(
                "FederationId {:?} not found in multimint map",
                federation_id
            );
            tracing::error!("{}", error_msg);
            AppError::new(StatusCode::BAD_REQUEST, anyhow::anyhow!(error_msg))
        })?;

        info!("Client found for federation ID: {:?}", federation_id);

        Ok((federation_id, client))
    }
}
