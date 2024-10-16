#![allow(dead_code)]
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlb::{bindable, Fields, HasFields};
use sqlx::FromRow;

use super::base::{self, DbBmc};
use super::ModelManager;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type)]
#[repr(i32)]
pub enum InvoiceState {
    /// The invoice is pending payment.
    Pending = 0,
    /// The invoice has been paid and settled.
    Settled = 1,
    /// The invoice has been cancelled or expired.
    Cancelled = 2,
}

bindable!(InvoiceState);

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Invoice {
    pub id: i32,
    pub federation_id: String,
    pub op_id: String,
    pub app_user_id: i32,
    pub bolt11: String,
    pub amount: i64,
    pub state: InvoiceState,
}

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct InvoiceForCreate {
    pub op_id: String,
    pub federation_id: String,
    pub app_user_id: i32,
    pub bolt11: String,
    pub amount: i64,
}

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct InvoiceForUpdate {
    pub state: InvoiceState,
}

pub struct InvoiceBmc;

impl DbBmc for InvoiceBmc {
    const TABLE: &'static str = "invoice";
}

impl InvoiceBmc {
    pub async fn create(mm: &ModelManager, inv_c: InvoiceForCreate) -> Result<i32> {
        base::create::<Self, _>(mm, inv_c).await
    }

    pub async fn get(mm: &ModelManager, id: i32) -> Result<Invoice> {
        base::get::<Self, _>(mm, id).await
    }

    pub async fn get_by_op_id(mm: &ModelManager, op_id: &str) -> Result<Invoice> {
        let inv: Invoice = sqlb::select()
            .table(Self::TABLE)
            .columns(Invoice::field_names())
            .and_where("op_id", "=", op_id)
            .fetch_optional(mm.db())
            .await?
            .ok_or(anyhow!("No invoice found with op_id: {}", op_id))?;
        Ok(inv)
    }

    /// Get all pending invoices
    pub async fn get_pending(mm: &ModelManager) -> Result<Vec<Invoice>> {
        let rows = sqlb::select()
            .table(Self::TABLE)
            .columns(Invoice::field_names())
            .and_where("state", "=", InvoiceState::Pending)
            .fetch_all(mm.db())
            .await?;

        Ok(rows)
    }

    pub async fn set_state(mm: &ModelManager, id: i32, state: InvoiceState) -> Result<Invoice> {
        let inv_u = InvoiceForUpdate { state };
        base::update::<Self, _>(mm, id, inv_u).await?;
        Self::get(mm, id).await
    }

    pub async fn delete(mm: &ModelManager, id: i32) -> Result<()> {
        base::delete::<Self>(mm, id).await
    }
}
