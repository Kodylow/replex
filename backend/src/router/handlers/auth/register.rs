use crate::{error::AppError, state::AppState};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterRequest {
    // TODO: Add fields for registration
}

#[derive(Serialize)]
pub struct RegisterResponse {
    // TODO: Add fields for response
}

#[axum_macros::debug_handler]
pub async fn handle_register(
    State(_state): State<AppState>,
    Json(_payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    todo!()
}
