use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::LnurlStatus;
use crate::error::AppError;
use crate::model::invoices::InvoiceState;
use crate::state::AppState;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LnurlVerifyResponse {
    pub status: LnurlStatus,
    pub settled: bool,
    pub preimage: String,
    pub pr: String,
}

#[axum_macros::debug_handler]
pub async fn handle_verify(
    Path((username, op_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<LnurlVerifyResponse>, AppError> {
    info!(
        "verify called with username: {}, op_id: {}",
        username, op_id
    );

    match state.db.invoice().get_by_op_id(&op_id).await? {
        Some(invoice) => {
            let verify_response = LnurlVerifyResponse {
                status: LnurlStatus::Ok,
                settled: invoice.state == InvoiceState::Settled,
                preimage: "".to_string(), // TODO: figure out how to get the preimage from fedimint client
                pr: invoice.bolt11,
            };
            info!("Verify response: {:?}", verify_response);

            Ok(Json(verify_response))
        }
        None => Err(AppError::new(
            StatusCode::NOT_FOUND,
            anyhow!("Invoice not found"),
        )),
    }
}
