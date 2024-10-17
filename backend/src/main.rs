pub mod config;
pub mod error;
pub mod model;
pub mod router;
pub mod state;
pub mod utils;

use config::CONFIG;

use anyhow::Result;
use state::AppState;
use tracing::{error, info};
use utils::{handle_pending_invoices, load_users_and_keys};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let state = AppState::new().await?;

    let app = router::create_router(state.clone()).await?;

    load_users_and_keys(state.clone()).await?;

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
