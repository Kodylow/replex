use crate::model::invoices::{Invoice, InvoiceState};
use crate::state::AppState;
use anyhow::Result;
use futures::StreamExt;
use multimint::fedimint_client::oplog::UpdateStreamOrOutcome;
use multimint::fedimint_core::config::FederationId;
use multimint::fedimint_core::task::spawn;
use multimint::fedimint_ln_client::{LightningClientModule, LnReceiveState};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{error, info};

use super::lnurl::notify_user;

pub async fn handle_pending_invoices(state: AppState) -> Result<()> {
    info!("Handling pending invoices");

    let invoices: Vec<Invoice> = state
        .db
        .query(
            "SELECT * FROM invoice WHERE state = $1",
            &[&InvoiceState::Pending],
        )
        .await?;

    let invoices_by_federation: HashMap<String, Vec<Invoice>> =
        invoices
            .into_iter()
            .fold(HashMap::new(), |mut acc, invoice| {
                acc.entry(invoice.federation_id.clone())
                    .or_default()
                    .push(invoice);
                acc
            });

    info!("Invoices by federation: {:?}", invoices_by_federation);

    for (federation_id, invoices) in invoices_by_federation {
        handle_federation_invoices(&state, &federation_id, invoices).await?;
    }

    Ok(())
}

async fn handle_federation_invoices(
    state: &AppState,
    federation_id: &str,
    invoices: Vec<Invoice>,
) -> Result<()> {
    info!(
        "Handling pending invoices for federation_id: {}",
        federation_id
    );

    match FederationId::from_str(federation_id) {
        Ok(federation_id) => {
            if let Some(client) = state.fm.clients.lock().await.get(&federation_id) {
                let ln = client.get_first_module::<LightningClientModule>();
                for invoice in invoices {
                    if let Ok(subscription) = ln
                        .subscribe_ln_receive(invoice.op_id.parse().expect("invalid op_id"))
                        .await
                    {
                        spawn_invoice_subscription(state.clone(), invoice, subscription).await?;
                    }
                }
            }
        }
        Err(e) => error!("Invalid federation_id: {}", e),
    }

    Ok(())
}

pub async fn spawn_invoice_subscription(
    state: AppState,
    invoice: Invoice,
    subscription: UpdateStreamOrOutcome<LnReceiveState>,
) -> Result<()> {
    spawn("waiting for invoice being paid", async move {
        let locked_clients = state.fm.clients.lock().await;
        let client = locked_clients
            .get(&FederationId::from_str(&invoice.federation_id).unwrap())
            .unwrap();
        let invoice_db = state.db.invoice();
        let mut stream = subscription.into_stream();
        while let Some(op_state) = stream.next().await {
            match op_state {
                LnReceiveState::Canceled { reason } => {
                    error!("Payment canceled, reason: {:?}", reason);
                    invoice_db
                        .update_state(invoice.id, InvoiceState::Cancelled)
                        .await
                        .expect("Failed to update invoice state");
                }
                LnReceiveState::Claimed => {
                    info!("Payment claimed");
                    invoice_db
                        .update_state(invoice.id, InvoiceState::Settled)
                        .await
                        .expect(&format!(
                            "Failed to update invoice state for invoice: {}",
                            invoice.id
                        ));
                    notify_user(client, &state.db, invoice)
                        .await
                        .expect("Failed to notify user");
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(())
}
