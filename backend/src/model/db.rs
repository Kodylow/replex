use std::collections::HashMap;

use anyhow::Result;
use deadpool_postgres::{Client, Pool, Runtime};
use postgres_from_row::FromRow;
use tokio_postgres::NoTls;
use tracing::info;

#[derive(Clone, Debug)]
pub struct Db(Pool);

impl Db {
    pub async fn new(db_url: String) -> Result<Db> {
        let connection_pool = {
            let mut pool_config = deadpool_postgres::Config::default();
            pool_config.url = Some(db_url);
            pool_config.create_pool(Some(Runtime::Tokio1), NoTls)
        }?;

        Ok(Db(connection_pool))
    }

    pub async fn client(&self) -> Result<Client> {
        let client = self.0.get().await?;
        Ok(client)
    }

    pub async fn setup_schema(&self) -> Result<()> {
        let client = self.client().await?;
        info!("Setting up schema");
        client
            .execute(include_str!("../../schema/v0.sql"), &[])
            .await?;
        info!("Schema setup complete");
        Ok(())
    }

    pub async fn execute(
        &self,
        sql: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> anyhow::Result<u64> {
        let client = self.client().await?;
        let num_rows = client.execute(sql, params).await?;
        Ok(num_rows)
    }

    pub async fn query_one<T>(
        &self,
        sql: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> anyhow::Result<T>
    where
        T: FromRow,
    {
        let client = self.client().await?;
        let result = client.query_one(sql, params).await?;
        Ok(T::try_from_row(&result)?)
    }

    pub async fn query_value<T>(
        &self,
        sql: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> anyhow::Result<T>
    where
        for<'a> T: tokio_postgres::types::FromSql<'a>,
    {
        let client = self.client().await?;
        let result = client.query_one(sql, params).await?;
        Ok(result.try_get(0)?)
    }

    pub async fn query_opt<T>(
        &self,
        sql: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> anyhow::Result<Option<T>>
    where
        T: FromRow,
    {
        let client = self.client().await?;
        let result = client.query_opt(sql, params).await?;
        Ok(result.map(|row| T::try_from_row(&row)).transpose()?)
    }

    pub async fn query<T>(
        &self,
        sql: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> anyhow::Result<Vec<T>>
    where
        T: FromRow,
    {
        let client = self.client().await?;
        let result = client.query(sql, params).await?;
        Ok(result
            .iter()
            .map(T::try_from_row)
            .collect::<Result<_, _>>()?)
    }

    pub async fn query_group_by<T>(
        &self,
        sql: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> anyhow::Result<HashMap<String, Vec<T>>>
    where
        T: FromRow,
    {
        let client = self.client().await?;
        let result = client.query(sql, params).await?;
        let mut grouped: HashMap<String, Vec<T>> = HashMap::new();
        for row in result {
            let item = T::try_from_row(&row)?;
            let key: String = row.try_get(0)?;
            grouped.entry(key).or_default().push(item);
        }
        Ok(grouped)
    }
}
