use crate::error::AppError;
use crate::model::users::User;
use anyhow::Result;
use axum::http::StatusCode;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::config::FederationId;
use multimint::MultiMint;
use std::str::FromStr;

pub async fn get_federation_and_client(
    mm: &MultiMint,
    user: &User,
) -> Result<(FederationId, ClientHandleArc), AppError> {
    let federation_id = FederationId::from_str(&user.federation_ids[0]).map_err(|e| {
        AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!("Invalid federation_id for user {}: {}", user.name, e),
        )
    })?;

    let locked_clients = mm.clients.lock().await.clone();
    let client = locked_clients.get(&federation_id).ok_or_else(|| {
        AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!("FederationId not found in multimint map"),
        )
    })?;

    Ok((federation_id, client.clone()))
}
