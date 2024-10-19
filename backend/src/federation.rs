use crate::error::AppError;
use crate::model::users::User;
use anyhow::Result;
use axum::http::StatusCode;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::config::FederationId;
use multimint::MultiMint;
use std::str::FromStr;
use tracing::info;

pub async fn get_federation_and_client(
    mm: &MultiMint,
    user: &User,
) -> Result<(FederationId, ClientHandleArc), AppError> {
    info!("Getting federation and client for user: {}", user.name);

    let federation_id = FederationId::from_str(&user.federation_ids[0]).map_err(|e| {
        let error_msg = format!("Invalid federation_id for user {}: {}", user.name, e);
        tracing::error!("{}", error_msg);
        AppError::new(StatusCode::BAD_REQUEST, anyhow::anyhow!(error_msg))
    })?;

    info!("Federation ID parsed: {:?}", federation_id);

    let locked_clients = mm.clients.lock().await.clone();
    let client = locked_clients.get(&federation_id).ok_or_else(|| {
        let error_msg = format!(
            "FederationId {:?} not found in multimint map",
            federation_id
        );
        tracing::error!("{}", error_msg);
        AppError::new(StatusCode::BAD_REQUEST, anyhow::anyhow!(error_msg))
    })?;

    info!("Client found for federation ID: {:?}", federation_id);

    Ok((federation_id, client.clone()))
}
