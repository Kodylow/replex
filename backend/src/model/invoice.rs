use anyhow::Result;
use multimint::{
    fedimint_core::{config::FederationId, core::OperationId},
    fedimint_ln_common::lightning_invoice::Bolt11Invoice,
};
use postgres_from_row::FromRow;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

use crate::error::AppError;

use super::db::Db;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum InvoiceState {
    /// The invoice is pending payment.
    Pending = 0,
    /// The invoice has been paid and settled.
    Settled = 1,
    /// The invoice has been cancelled or expired.
    Cancelled = 2,
}

impl FromSql<'_> for InvoiceState {
    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "invoice_state" || ty.name() == "int4"
    }

    fn from_sql(
        ty: &postgres_types::Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = i32::from_sql(ty, raw)?;
        match value {
            0 => Ok(InvoiceState::Pending),
            1 => Ok(InvoiceState::Settled),
            2 => Ok(InvoiceState::Cancelled),
            _ => Err(format!("Invalid invoice state: {}", value).into()),
        }
    }
}

impl ToSql for InvoiceState {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        (*self as i32).to_sql(ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "invoice_state" || ty.name() == "int4"
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        (*self as i32).to_sql_checked(ty, out)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Invoice {
    pub id: i32,
    pub federation_id: String,
    pub op_id: String,
    pub app_user_id: i32,
    pub bolt11: String,
    pub amount: i64,
    pub state: InvoiceState,
    pub tweak: i64,
}

impl FromRow for Invoice {
    fn from_row(row: &Row) -> Self {
        Self::try_from_row(row).expect("Decoding row failed")
    }

    fn try_from_row(row: &Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Invoice {
            id: row.get("id"),
            federation_id: row.get("federation_id"),
            op_id: row.get("op_id"),
            app_user_id: row.get("app_user_id"),
            bolt11: row.get("bolt11"),
            amount: row.get("amount"),
            state: row.get("state"),
            tweak: row.get("tweak"),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InvoiceForUpdate {
    pub state: InvoiceState,
}

impl InvoiceForUpdate {
    pub fn builder() -> InvoiceForUpdateBuilder {
        InvoiceForUpdateBuilder::default()
    }
}

#[derive(Default)]
pub struct InvoiceForUpdateBuilder {
    state: Option<InvoiceState>,
}

impl InvoiceForUpdateBuilder {
    pub fn state(mut self, state: InvoiceState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn build(self) -> Result<InvoiceForUpdate> {
        Ok(InvoiceForUpdate {
            state: self.state.expect("state is required"),
        })
    }
}

pub async fn insert_invoice(
    db: &Db,
    federation_id: &FederationId,
    user_id: i32,
    op_id: &OperationId,
    invoice: &Bolt11Invoice,
    tweak: i64,
    amount: u64,
) -> Result<Invoice, AppError> {
    let sql = "INSERT INTO invoices (op_id, federation_id, app_user_id, amount, bolt11, tweak, state) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *";
    let invoice = Invoice {
        id: 0,
        op_id: op_id.fmt_full().to_string(),
        federation_id: federation_id.to_string(),
        app_user_id: user_id,
        amount: amount as i64,
        bolt11: invoice.to_string(),
        tweak,
        state: InvoiceState::Pending,
    };
    db.query_one::<Invoice>(
        sql,
        &[
            &invoice.op_id,
            &invoice.federation_id,
            &invoice.app_user_id,
            &invoice.amount,
            &invoice.bolt11,
            &invoice.tweak,
            &invoice.state,
        ],
    )
    .await
    .map_err(|e| e.into())
}

pub async fn update_invoice_state(db: &Db, op_id: &str, state: InvoiceState) -> Result<()> {
    let sql = "UPDATE invoices SET state = $1 WHERE op_id = $2";
    let _ = db.execute(sql, &[&state, &op_id]).await?;
    Ok(())
}
