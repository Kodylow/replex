pub mod db;

use anyhow::Result;
use postgres_from_row::FromRow;
use serde::Serialize;
use tokio_postgres::Row;

#[derive(Debug, Clone, Serialize, Default)]
pub struct UserForCreate {
    pub name: String,
    pub pubkey: String,
    pub last_tweak: i64,
    pub relays: Vec<String>,
    pub federation_ids: Vec<String>,
}

impl UserForCreate {
    pub fn builder() -> UserForCreateBuilder {
        UserForCreateBuilder::default()
    }
}

#[derive(Default)]
pub struct UserForCreateBuilder {
    name: Option<String>,
    pubkey: Option<String>,
    last_tweak: Option<i64>,
    relays: Option<Vec<String>>,
    federation_ids: Option<Vec<String>>,
}

impl UserForCreateBuilder {
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

    pub fn build(self) -> anyhow::Result<UserForCreate> {
        Ok(UserForCreate {
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
pub struct User {
    pub id: i32,
    pub name: String,
    pub pubkey: String,
    pub last_tweak: i64,
    pub relays: Vec<String>,
    pub federation_ids: Vec<String>,
}

impl FromRow for User {
    fn from_row(row: &Row) -> Self {
        Self::try_from_row(row).expect("Decoding row failed")
    }

    fn try_from_row(row: &Row) -> Result<Self, tokio_postgres::Error> {
        Ok(User {
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
pub struct UserForUpdate {
    pub pubkey: Option<String>,
    pub name: Option<String>,
    pub relays: Option<Vec<String>>,
    pub federation_ids: Option<Vec<String>>,
    pub last_tweak: Option<i64>,
}

impl UserForUpdate {
    pub fn builder() -> UserForUpdateBuilder {
        UserForUpdateBuilder::default()
    }
}

#[derive(Default)]
pub struct UserForUpdateBuilder {
    pubkey: Option<String>,
    name: Option<String>,
    relays: Option<Vec<String>>,
    federation_ids: Option<Vec<String>>,
    last_tweak: Option<i64>,
}

impl UserForUpdateBuilder {
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

    pub fn build(self) -> UserForUpdate {
        UserForUpdate {
            pubkey: self.pubkey,
            name: self.name,
            relays: self.relays,
            federation_ids: self.federation_ids,
            last_tweak: self.last_tweak,
        }
    }
}
