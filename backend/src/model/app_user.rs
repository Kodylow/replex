use anyhow::Result;
use postgres_from_row::FromRow;
use serde::Serialize;
use tokio_postgres::Row;

use super::db::Db;

#[derive(Debug, Clone, Serialize, Default)]
pub struct AppUserForCreate {
    pub name: String,
    pub pubkey: String,
    pub last_tweak: i64,
    pub relays: Vec<String>,
    pub federation_ids: Vec<String>,
}

impl AppUserForCreate {
    pub fn builder() -> AppUserForCreateBuilder {
        AppUserForCreateBuilder::default()
    }
}

#[derive(Default)]
pub struct AppUserForCreateBuilder {
    name: Option<String>,
    pubkey: Option<String>,
    last_tweak: Option<i64>,
    relays: Option<Vec<String>>,
    federation_ids: Option<Vec<String>>,
}

impl AppUserForCreateBuilder {
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn pubkey(mut self, pubkey: String) -> Self {
        self.pubkey = Some(pubkey);
        self
    }

    pub fn last_tweak(mut self, last_tweak: i64) -> Self {
        self.last_tweak = Some(last_tweak);
        self
    }

    pub fn relays(mut self, relays: Vec<String>) -> Self {
        self.relays = Some(relays);
        self
    }

    pub fn federation_ids(mut self, federation_ids: Vec<String>) -> Self {
        self.federation_ids = Some(federation_ids);
        self
    }

    pub fn build(self) -> anyhow::Result<AppUserForCreate> {
        Ok(AppUserForCreate {
            name: self
                .name
                .ok_or_else(|| anyhow::anyhow!("name is required"))?,
            pubkey: self
                .pubkey
                .ok_or_else(|| anyhow::anyhow!("pubkey is required"))?,
            last_tweak: self
                .last_tweak
                .ok_or_else(|| anyhow::anyhow!("last_tweak is required"))?,
            relays: self
                .relays
                .ok_or_else(|| anyhow::anyhow!("relays is required"))?,
            federation_ids: self
                .federation_ids
                .ok_or_else(|| anyhow::anyhow!("federation_ids is required"))?,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AppUser {
    pub id: i32,
    pub name: String,
    pub pubkey: String,
    pub last_tweak: i64,
    pub relays: Vec<String>,
    pub federation_ids: Vec<String>,
}

impl FromRow for AppUser {
    fn from_row(row: &Row) -> Self {
        Self::try_from_row(row).expect("Decoding row failed")
    }

    fn try_from_row(row: &Row) -> Result<Self, tokio_postgres::Error> {
        Ok(AppUser {
            id: row.get("id"),
            name: row.get("name"),
            pubkey: row.get("pubkey"),
            last_tweak: row.get("last_tweak"),
            relays: row.get::<_, Vec<String>>("relays"),
            federation_ids: row
                .get::<_, Vec<String>>("federation_ids")
                .iter()
                .map(|s| s.to_string())
                .collect(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AppUserForUpdate {
    pub pubkey: Option<String>,
    pub name: Option<String>,
    pub relays: Option<Vec<String>>,
    pub federation_ids: Option<Vec<String>>,
    pub last_tweak: Option<i64>,
}

impl AppUserForUpdate {
    pub fn builder() -> AppUserForUpdateBuilder {
        AppUserForUpdateBuilder::default()
    }
}

#[derive(Default)]
pub struct AppUserForUpdateBuilder {
    pubkey: Option<String>,
    name: Option<String>,
    relays: Option<Vec<String>>,
    federation_ids: Option<Vec<String>>,
    last_tweak: Option<i64>,
}

impl AppUserForUpdateBuilder {
    pub fn pubkey(mut self, pubkey: String) -> Self {
        self.pubkey = Some(pubkey);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn relays(mut self, relays: Vec<String>) -> Self {
        self.relays = Some(relays);
        self
    }

    pub fn federation_ids(mut self, federation_ids: Vec<String>) -> Self {
        self.federation_ids = Some(federation_ids);
        self
    }

    pub fn last_tweak(mut self, last_tweak: i64) -> Self {
        self.last_tweak = Some(last_tweak);
        self
    }

    pub fn build(self) -> AppUserForUpdate {
        AppUserForUpdate {
            pubkey: self.pubkey,
            name: self.name,
            relays: self.relays,
            federation_ids: self.federation_ids,
            last_tweak: self.last_tweak,
        }
    }
}

impl AppUser {
    pub async fn update_tweak(db: &Db, user: &AppUser, tweak: i64) -> Result<()> {
        let sql = "UPDATE app_user SET last_tweak = $1 WHERE id = $2";
        db.execute(sql, &[&tweak, &user.id]).await?;
        Ok(())
    }
}

pub async fn get_user(db: &Db, username: &str) -> Result<AppUser> {
    let sql = "SELECT * FROM app_user WHERE name = $1";
    db.query_one::<AppUser>(sql, &[&username])
        .await
        .map_err(|e| e.into())
}

pub struct UserDb(pub(crate) Db);

impl UserDb {
    pub async fn create(&self, user: AppUserForCreate) -> Result<AppUser> {
        let sql = "INSERT INTO app_user (name, pubkey, last_tweak, relays, federation_ids) VALUES ($1, $2, $3, $4, $5) RETURNING *";
        self.0
            .query_one::<AppUser>(
                sql,
                &[
                    &user.name,
                    &user.pubkey,
                    &user.last_tweak,
                    &user.relays,
                    &user.federation_ids,
                ],
            )
            .await
    }

    pub async fn get(&self, username: &str) -> Result<AppUser> {
        let sql = "SELECT * FROM app_user WHERE name = $1";
        self.0.query_one::<AppUser>(sql, &[&username]).await
    }

    pub async fn update(&self, id: i32, user: AppUserForUpdate) -> Result<AppUser> {
        let mut updates = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        let mut param_count = 1;

        if let Some(name) = &user.name {
            updates.push(format!("name = ${}", param_count));
            params.push(name);
            param_count += 1;
        }
        // ... repeat for other fields ...

        let sql = format!(
            "UPDATE app_user SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_count
        );
        params.push(&id);

        self.0.query_one::<AppUser>(&sql, &params).await
    }

    pub async fn update_tweak(&self, user: &AppUser, tweak: i64) -> Result<()> {
        let sql = "UPDATE app_user SET last_tweak = $1 WHERE id = $2";
        self.0.execute(sql, &[&tweak, &user.id]).await?;
        Ok(())
    }
}
