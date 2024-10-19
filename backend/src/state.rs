use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use config::CONFIG;
use futures::StreamExt;
use multimint::{
    fedimint_client::{oplog::UpdateStreamOrOutcome, ClientHandleArc},
    fedimint_core::config::FederationId,
    fedimint_ln_client::{LightningClientModule, LnReceiveState},
    MultiMint,
};
use tokio::task::spawn;
use tracing::{error, info};

use crate::{
    config,
    model::{
        invoices::{Invoice, InvoiceState},
        Db,
    },
    nostr::Nostr,
};

#[derive(Clone)]
pub struct AppState {
    pub mm: MultiMint,
    pub db: Db,
    pub nostr: Nostr,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let mut mm = MultiMint::new(CONFIG.fm_db_path.clone()).await?;
        for invite_code in CONFIG.federation_invite_codes.iter() {
            info!("Registering federation invite code: {}", invite_code);
            mm.register_new(invite_code.clone(), None).await?;
        }
        let db = Db::new(CONFIG.pg_db.clone()).await?;
        let nostr = Nostr::new(&CONFIG.nostr_nsec)?;
        nostr.add_relays(&CONFIG.nostr_relays).await?;
        db.setup_schema().await?;

        Ok(Self { mm, db, nostr })
    }

    pub async fn handle_pending_invoices(&self) -> Result<()> {
        info!("Handling pending invoices");

        let pending_invoices: Vec<Invoice> = self
            .db
            .invoice()
            .get_by_state(InvoiceState::Pending)
            .await?;

        let pending_invoices_by_federation: HashMap<String, Vec<Invoice>> = pending_invoices
            .into_iter()
            .fold(HashMap::new(), |mut acc, invoice| {
                acc.entry(invoice.federation_id.clone())
                    .or_default()
                    .push(invoice);
                acc
            });

        info!(
            "Pending invoices by federation: {:?}",
            pending_invoices_by_federation
        );

        for (federation_id, invoices) in pending_invoices_by_federation {
            self.handle_federation_invoices(&federation_id, invoices)
                .await?;
        }

        Ok(())
    }

    async fn handle_federation_invoices(
        &self,
        federation_id: &str,
        invoices: Vec<Invoice>,
    ) -> Result<()> {
        info!(
            "Handling pending invoices for federation_id: {}",
            federation_id
        );

        match FederationId::from_str(federation_id) {
            Ok(federation_id) => {
                if let Some(client) = self.mm.clients.lock().await.get(&federation_id).cloned() {
                    let ln = client.get_first_module::<LightningClientModule>();
                    for invoice in invoices {
                        info!("Processing invoice: {:?}", invoice);
                        if let Ok(subscription) = ln
                            .subscribe_ln_receive(invoice.op_id.parse().expect("invalid op_id"))
                            .await
                        {
                            info!("Successfully subscribed to invoice: {}", invoice.op_id);
                            self.spawn_invoice_subscription(invoice, subscription)
                                .await?;
                        } else {
                            error!("Failed to subscribe to invoice: {}", invoice.op_id);
                        }
                    }
                } else {
                    error!("No client found for federation_id: {}", federation_id);
                }
            }
            Err(e) => error!("Invalid federation_id: {}", e),
        }

        Ok(())
    }

    pub async fn spawn_invoice_subscription(
        &self,
        invoice: Invoice,
        subscription: UpdateStreamOrOutcome<LnReceiveState>,
    ) -> Result<()> {
        let invoice_db = self.db.invoice();
        let state = self.clone();
        spawn(async move {
            info!("Spawned task for invoice: {}", invoice.op_id);
            let mut stream = subscription.into_stream();
            while let Some(op_state) = stream.next().await {
                info!(
                    "Received state update for invoice {}: {:?}",
                    invoice.op_id, op_state
                );
                match op_state {
                    LnReceiveState::Canceled { reason } => {
                        error!(
                            "Payment canceled for invoice {}, reason: {:?}",
                            invoice.op_id, reason
                        );
                        invoice_db
                            .update_state(invoice.id, InvoiceState::Cancelled)
                            .await
                            .expect("Failed to update invoice state");
                    }
                    LnReceiveState::Claimed => {
                        info!("Payment claimed for invoice {}", invoice.op_id);
                        invoice_db
                            .update_state(invoice.id, InvoiceState::Settled)
                            .await
                            .expect(&format!(
                                "Failed to update invoice state for invoice: {}",
                                invoice.id
                            ));
                        match state.db.users().get(invoice.user_id).await {
                            Ok(Some(user)) => {
                                info!(
                                    "Notifying user {} for settled invoice {}",
                                    user.id, invoice.op_id
                                );
                                state
                                    .nostr
                                    .notify_user(&user, invoice)
                                    .await
                                    .expect("Failed to notify user");
                            }
                            Ok(None) => error!("User not found for invoice: {}", invoice.id),
                            Err(e) => error!("Failed to get user for invoice: {}", e),
                        }
                        break;
                    }
                    _ => {
                        info!(
                            "Unhandled state for invoice {}: {:?}",
                            invoice.op_id, op_state
                        );
                    }
                }
            }
        });

        Ok(())
    }
}
