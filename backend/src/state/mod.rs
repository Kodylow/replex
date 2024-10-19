pub mod invoice_manager;

use std::str::FromStr;

use anyhow::Result;
use axum::http::StatusCode;
use config::CONFIG;
use invoice_manager::InvoiceManager;
use multimint::{
    fedimint_client::ClientHandleArc,
    fedimint_core::{config::FederationId, core::OperationId},
    fedimint_ln_client::LightningClientModule,
    fedimint_ln_common::lightning_invoice::Bolt11Invoice,
    MultiMint,
};
use tracing::info;

use crate::{
    config,
    error::AppError,
    lnurl::create_invoice,
    model::{
        invoices::{InvoiceForCreate, InvoiceState},
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
    pub invoice_manager: InvoiceManager,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let mm = MultiMint::new(CONFIG.fm_db_path.clone()).await?;
        let db = Db::new(CONFIG.pg_db.clone()).await?;
        let nostr = Nostr::new(&CONFIG.nostr_nsec)?;

        nostr.add_relays(&CONFIG.nostr_relays).await?;
        db.setup_schema().await?;

        let invoice_manager = InvoiceManager::new(db.invoice());

        Ok(Self {
            mm,
            db,
            nostr,
            invoice_manager,
        })
    }

    pub async fn register_federations(&mut self) -> Result<()> {
        for invite_code in &CONFIG.federation_invite_codes {
            self.mm.register_new(invite_code.clone(), None).await?;
        }
        Ok(())
    }

    pub async fn handle_pending_invoices(&self) -> Result<()> {
        self.invoice_manager.handle_pending_invoices(&self.mm).await
    }

    pub async fn create_and_store_invoice(
        &self,
        username: &str,
        params: &LnurlCallbackParams,
    ) -> Result<(Bolt11Invoice, OperationId), AppError> {
        let user = self.db.users().get_by_name(username).await?.unwrap();
        let (federation_id, client) = self.get_federation_and_client(&user).await?;
        let ln = client.get_first_module::<LightningClientModule>();

        let tweak = user.last_tweak + 1;
        let (op_id, invoice, _) = create_invoice(&ln, params, &user, tweak).await?;

        let stored_invoice = self
            .db
            .invoice()
            .create(InvoiceForCreate {
                op_id: op_id.fmt_full().to_string(),
                federation_id: federation_id.to_string(),
                user_id: user.id,
                amount: params.amount as i64,
                bolt11: invoice.to_string(),
                tweak,
                state: InvoiceState::Pending,
            })
            .await?;

        self.invoice_manager
            .subscribe_to_invoice(&ln, stored_invoice)
            .await?;

        Ok((invoice, op_id))
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

        let locked_clients = self.mm.clients.lock().await.clone();
        let client = locked_clients.get(&federation_id).ok_or_else(|| {
            let error_msg = format!(
                "FederationId {:?} not found in multimint map",
                federation_id
            );
            tracing::error!("{}", error_msg);
            AppError::new(StatusCode::BAD_REQUEST, anyhow::anyhow!(error_msg))
        })?;

        info!("Client found for federation ID: {:?}", federation_id);

        Ok((federation_id, client.clone()))
    }
}
