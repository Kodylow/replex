pub mod db;

use anyhow::Result;
use postgres_from_row::FromRow;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum InvoiceState {
    Pending = 0,
    Settled = 1,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceForCreate {
    pub op_id: String,
    pub federation_id: String,
    pub app_user_id: i32,
    pub amount: i64,
    pub bolt11: String,
    pub tweak: i64,
    pub state: InvoiceState,
}

impl InvoiceForCreate {
    pub fn builder() -> InvoiceForCreateBuilder {
        InvoiceForCreateBuilder::default()
    }
}

#[derive(Default)]
pub struct InvoiceForCreateBuilder {
    op_id: Option<String>,
    federation_id: Option<String>,
    app_user_id: Option<i32>,
    amount: Option<i64>,
    bolt11: Option<String>,
    tweak: Option<i64>,
    state: Option<InvoiceState>,
}

impl InvoiceForCreateBuilder {
    pub fn op_id(mut self, op_id: String) -> Self {
        self.op_id = Some(op_id);
        self
    }

    pub fn federation_id(mut self, federation_id: String) -> Self {
        self.federation_id = Some(federation_id);
        self
    }

    pub fn app_user_id(mut self, app_user_id: i32) -> Self {
        self.app_user_id = Some(app_user_id);
        self
    }

    pub fn amount(mut self, amount: i64) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn bolt11(mut self, bolt11: String) -> Self {
        self.bolt11 = Some(bolt11);
        self
    }

    pub fn tweak(mut self, tweak: i64) -> Self {
        self.tweak = Some(tweak);
        self
    }

    pub fn state(mut self, state: InvoiceState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn build(self) -> anyhow::Result<InvoiceForCreate> {
        Ok(InvoiceForCreate {
            op_id: self
                .op_id
                .ok_or_else(|| anyhow::anyhow!("op_id is required"))?,
            federation_id: self
                .federation_id
                .ok_or_else(|| anyhow::anyhow!("federation_id is required"))?,
            app_user_id: self
                .app_user_id
                .ok_or_else(|| anyhow::anyhow!("app_user_id is required"))?,
            amount: self
                .amount
                .ok_or_else(|| anyhow::anyhow!("amount is required"))?,
            bolt11: self
                .bolt11
                .ok_or_else(|| anyhow::anyhow!("bolt11 is required"))?,
            tweak: self
                .tweak
                .ok_or_else(|| anyhow::anyhow!("tweak is required"))?,
            state: self
                .state
                .ok_or_else(|| anyhow::anyhow!("state is required"))?,
        })
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
