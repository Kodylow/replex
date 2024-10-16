pub mod config;
pub mod error;
pub mod model;
pub mod router;
pub mod state;
pub mod utils;

use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;
use config::CONFIG;
use itertools::Itertools;
use model::invoice::InvoiceBmc;
use multimint::{fedimint_core::config::FederationId, fedimint_ln_client::LightningClientModule};
use router::handlers::lnurlp::callback::spawn_invoice_subscription;
use state::AppState;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let state = AppState::new().await?;

    let app = router::create_router(state.clone()).await?;

    // spawn a task to check for previous pending invoices
    tokio::spawn(async move {
        if let Err(e) = handle_pending_invoices(state).await {
            error!("Error handling pending invoices: {e}")
        }
    });

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", CONFIG.domain, CONFIG.port))
        .await
        .unwrap();
    info!("Listening on {}", CONFIG.port);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

/// Starts subscription for all pending invoices from previous run
async fn handle_pending_invoices(state: AppState) -> Result<()> {
    let invoices = InvoiceBmc::get_pending(&state.mm).await?;

    // Group invoices by federation_id
    let invoices_by_federation = invoices
        .into_iter()
        .chunk_by(|i| i.federation_id.clone())
        .into_iter()
        .map(|(federation_id, invs)| (federation_id, invs.collect::<Vec<_>>()))
        .collect::<HashMap<_, _>>();

    for (federation_id, invoices) in invoices_by_federation {
        // Get the corresponding multimint client for the federation_id
        if let Ok(federation_id) = FederationId::from_str(&federation_id) {
            if let Some(client) = state.fm.clients.lock().await.get(&federation_id) {
                let ln = client.get_first_module::<LightningClientModule>();
                for invoice in invoices {
                    // Create subscription to operation if it exists
                    if let Ok(subscription) = ln
                        .subscribe_ln_receive(invoice.op_id.parse().expect("invalid op_id"))
                        .await
                    {
                        let nip05relays =
                            AppUserRelaysBmc::get_by_id(&state.mm, invoice.app_user_id).await?;
                        spawn_invoice_subscription(
                            state.clone(),
                            invoice.id,
                            nip05relays.clone(),
                            subscription,
                        )
                        .await;
                    }
                }
            }
        }
    }

    Ok(())
}
