use anyhow::Result;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use url::Url;

use super::LnurlStatus;
use crate::error::AppError;
use crate::lnurl::validate_amount;
use crate::serde_helpers::empty_string_as_none;
use crate::state::AppState;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LnurlCallbackParams {
    pub amount: u64,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub nonce: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub comment: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub proofofpayer: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub nostr: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LnurlCallbackSuccessAction {
    pub tag: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LnurlCallbackResponse {
    pub status: LnurlStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub pr: String,
    pub verify: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_action: Option<LnurlCallbackSuccessAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routes: Option<Vec<String>>,
}

#[axum_macros::debug_handler]
pub async fn handle_callback(
    Path(username): Path<String>,
    Query(params): Query<LnurlCallbackParams>,
    State(state): State<AppState>,
) -> Result<Json<LnurlCallbackResponse>, AppError> {
    debug!("Callback for user: {}, params: {:?}", username, params);
    validate_amount(params.amount)?;

    let (invoice, op_id) = state.create_and_store_invoice(&username, &params).await?;

    let verify_url = create_verify_url(&username, &op_id.fmt_full().to_string())?;
    let response = create_callback_response(invoice.to_string(), verify_url)?;

    info!(
        "Callback processed for user: {}, op_id: {:?}",
        username, op_id
    );
    Ok(Json(response))
}

pub fn create_verify_url(username: &str, op_id: &str) -> Result<Url, AppError> {
    Url::parse(&format!(
        "http://{}:{}/lnurlp/{}/verify/{}",
        crate::config::CONFIG.domain,
        crate::config::CONFIG.port,
        username,
        op_id
    ))
    .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow::anyhow!(e)))
}

pub fn create_callback_response(
    bolt11: String,
    verify_url: Url,
) -> Result<LnurlCallbackResponse, AppError> {
    Ok(LnurlCallbackResponse {
        pr: bolt11,
        success_action: None,
        status: LnurlStatus::Ok,
        reason: None,
        verify: verify_url,
        routes: Some(vec![]),
    })
}
