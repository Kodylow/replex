pub mod config;
pub mod error;
pub mod model;
pub mod nostr;
pub mod router;
pub mod serde_helpers;
pub mod state;

use config::CONFIG;

use anyhow::Result;
use state::AppState;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let state = AppState::new().await?;

    let app = router::create_router(state.clone()).await?;

    // spawn a task to check for previous pending invoices
    tokio::spawn(async move {
        if let Err(e) = state.handle_pending_invoices().await {
            error!("Error handling pending invoices: {e}")
        }
    });
    info!("Started pending invoice handler");

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", CONFIG.domain, CONFIG.port))
        .await
        .unwrap();
    info!("Listening on {}", CONFIG.port);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
