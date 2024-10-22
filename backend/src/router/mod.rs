use anyhow::Result;
use axum::routing::get;
use axum::Router;
pub mod handlers;

use handlers::{handle_home, invoices, lnurlp};

use crate::state::AppState;

pub async fn create_router(state: AppState) -> Result<Router> {
    let app = Router::new()
        .route("/", get(handle_home))
        .route("/health", get(|| async { "OK" }))
        .route("/invoices", get(invoices::handle_invoices))
        .route(
            "/.well-known/lnurlp/:username",
            get(lnurlp::well_known::handle_well_known),
        )
        .route(
            "/lnurlp/:username/callback",
            get(lnurlp::callback::handle_callback),
        )
        .route(
            "/lnurlp/:username/verify/:op_id",
            get(lnurlp::verify::handle_verify),
        )
        .with_state(state);

    Ok(app)
}
