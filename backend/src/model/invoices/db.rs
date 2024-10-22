use crate::model::Db;
use anyhow::Result;

use super::{Invoice, InvoiceForCreate, InvoiceState};

#[derive(Clone)]
pub struct InvoiceDb(pub Db);

impl InvoiceDb {
    pub async fn create(&self, invoice: InvoiceForCreate) -> Result<Invoice> {
        let sql = "INSERT INTO invoices (op_id, federation_id, user_id, user_pubkey, amount, bolt11, tweak, state) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *";
        self.0
            .query_one::<Invoice>(
                sql,
                &[
                    &invoice.op_id,
                    &invoice.federation_id,
                    &invoice.user_id,
                    &invoice.user_pubkey,
                    &invoice.amount,
                    &invoice.bolt11,
                    &invoice.tweak,
                    &invoice.state,
                ],
            )
            .await
    }

    // Id on invoice is the operation id from the fedimint client
    pub async fn get_by_op_id(&self, op_id: &str) -> Result<Option<Invoice>> {
        let sql = "SELECT * FROM invoices WHERE op_id = $1";
        self.0.query_opt::<Invoice>(sql, &[&op_id]).await
    }

    pub async fn update_state(&self, id: i32, state: InvoiceState) -> Result<()> {
        let sql = "UPDATE invoices SET state = $1 WHERE id = $2";
        let _ = self.0.execute(sql, &[&state, &id]).await?;
        Ok(())
    }

    pub async fn get_by_state(&self, state: InvoiceState) -> Result<Vec<Invoice>> {
        let sql = "SELECT * FROM invoices WHERE state = $1";
        self.0.query(sql, &[&state]).await
    }
}
