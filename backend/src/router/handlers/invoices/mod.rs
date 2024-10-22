use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use crate::state::AppState;
use crate::error::AppError;
use crate::model::invoices::{Invoice, InvoiceState};

#[derive(Deserialize)]
pub struct InvoiceQuery {
    user_id: i32,
}

pub async fn handle_invoices(
    Query(query): Query<InvoiceQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Invoice>>, AppError> {
    let invoices = state.db.invoices().get_by_state(InvoiceState::Pending).await?;
    let user_invoices = invoices.into_iter()
        .filter(|invoice| invoice.user_id == query.user_id)
        .collect::<Vec<Invoice>>();
    
    Ok(Json(user_invoices))
}
