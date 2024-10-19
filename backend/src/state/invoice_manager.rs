use anyhow::Result;
use futures::StreamExt;
use multimint::{
    fedimint_client::oplog::UpdateStreamOrOutcome,
    fedimint_core::config::FederationId,
    fedimint_ln_client::{LightningClientModule, LnReceiveState},
    MultiMint,
};
use std::{collections::HashMap, str::FromStr};
use tokio::task::spawn;
use tracing::{error, info};

use crate::model::invoices::{db::InvoiceDb, Invoice, InvoiceState};

#[derive(Clone)]
pub struct InvoiceManager {
    invoice_db: InvoiceDb,
}

impl InvoiceManager {
    pub fn new(invoice_db: InvoiceDb) -> Self {
        Self { invoice_db }
    }

    pub async fn handle_pending_invoices(&self, mm: &MultiMint) -> Result<()> {
        let pending_invoices = self.invoice_db.get_by_state(InvoiceState::Pending).await?;
        let pending_invoices_by_federation = self.group_invoices_by_federation(pending_invoices);

        for (federation_id, invoices) in pending_invoices_by_federation {
            self.handle_federation_invoices(mm, &federation_id, invoices)
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
        mm: &MultiMint,
        federation_id: &str,
        invoices: Vec<Invoice>,
    ) -> Result<()> {
        info!(
            "Processing {} invoices for federation: {}",
            invoices.len(),
            federation_id
        );

        let federation_id = FederationId::from_str(federation_id).map_err(|e| {
            error!("Invalid federation ID: {}", e);
            anyhow::anyhow!("Invalid federation ID")
        })?;

        let client = mm
            .clients
            .lock()
            .await
            .get(&federation_id)
            .cloned()
            .ok_or_else(|| {
                error!("Client not found for federation: {}", federation_id);
                anyhow::anyhow!("Client not found")
            })?;

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
        let op_id = invoice.op_id.parse().expect("invalid op_id");
        let subscription = ln.subscribe_ln_receive(op_id).await.map_err(|e| {
            error!("Failed to subscribe to invoice: {}", e);
            anyhow::anyhow!("Failed to subscribe to invoice")
        })?;

        self.spawn_invoice_subscription(invoice, subscription).await
    }

    async fn spawn_invoice_subscription(
        &self,
        invoice: Invoice,
        subscription: UpdateStreamOrOutcome<LnReceiveState>,
    ) -> Result<()> {
        let invoice_db = self.invoice_db.clone();

        spawn(async move {
            info!("Monitoring invoice: {}", invoice.op_id);
            let mut stream = subscription.into_stream();
            while let Some(op_state) = stream.next().await {
                match op_state {
                    LnReceiveState::Canceled { reason } => {
                        error!("Invoice {} canceled: {:?}", invoice.op_id, reason);
                        invoice_db
                            .update_state(invoice.id, InvoiceState::Cancelled)
                            .await
                            .expect("Failed to update invoice state");
                    }
                    LnReceiveState::Claimed => {
                        info!("Invoice {} claimed", invoice.op_id);
                        invoice_db
                            .update_state(invoice.id, InvoiceState::Settled)
                            .await
                            .expect("Failed to update invoice state");
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}
