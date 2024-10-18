use crate::error::AppError;
use crate::model::Db;
use anyhow::Result;

use super::{Invoice, InvoiceForCreate, InvoiceState};

pub struct InvoiceDb(pub Db);

impl InvoiceDb {
    pub async fn create(&self, invoice: InvoiceForCreate) -> Result<Invoice, AppError> {
        let sql = "INSERT INTO invoices (op_id, federation_id, app_user_id, amount, bolt11, tweak, state) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *";
        self.0
            .query_one::<Invoice>(
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

    pub async fn update_state(&self, id: i32, state: InvoiceState) -> Result<()> {
        let sql = "UPDATE invoices SET state = $1 WHERE id = $2";
        let _ = self.0.execute(sql, &[&state, &id]).await?;
        Ok(())
    }

    // Add more methods as needed, e.g., get, list, etc.
}
