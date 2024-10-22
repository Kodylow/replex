pub mod db;

use anyhow::Result;
use postgres_from_row::FromRow;
use serde::Serialize;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct UserForCreate {
    pub name: String,
    pub replit_id: String,
    pub replit_profile_pic: String,
    pub pubkey: String,
    pub relays: Vec<String>,
    pub federation_ids: Vec<String>,
    pub connection_code_uuid: String,
    pub last_tweak: i64,
}

impl UserForCreate {
    pub fn new(
        name: String,
        replit_id: String,
        replit_profile_pic: String,
        pubkey: String,
        relays: Vec<String>,
        federation_ids: Vec<String>,
    ) -> Self {
        Self {
            name,
            replit_id,
            replit_profile_pic,
            pubkey,
            relays,
            federation_ids,
            connection_code_uuid: Uuid::new_v4().to_string(),
            last_tweak: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub replit_id: String,
    pub replit_profile_pic: String,
    pub pubkey: String,
    pub last_tweak: i64,
    pub relays: Vec<String>,
    pub federation_ids: Vec<String>,
    pub connection_code_uuid: String,
}

impl FromRow for User {
    fn from_row(row: &Row) -> Self {
        Self::try_from_row(row).expect("Decoding row failed")
    }

    fn try_from_row(row: &Row) -> Result<Self, tokio_postgres::Error> {
        Ok(User {
            id: row.get("id"),
            name: row.get("name"),
            replit_id: row.get("replit_id"),
            replit_profile_pic: row.get("replit_profile_pic"),
            pubkey: row.get("pubkey"),
            last_tweak: row.get("last_tweak"),
            relays: row.get("relays"),
            federation_ids: row.get("federation_ids"),
            connection_code_uuid: row.get("connection_code"),
        })
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UserForUpdate {
    pub name: Option<String>,
    pub replit_id: Option<String>,
    pub replit_profile_pic: Option<String>,
    pub pubkey: Option<String>,
    pub relays: Option<Vec<String>>,
    pub federation_ids: Option<Vec<String>>,
    pub last_tweak: Option<i64>,
    pub connection_code_uuid: Option<String>,
}

impl UserForUpdate {
    pub fn builder() -> UserForUpdateBuilder {
        UserForUpdateBuilder::default()
    }
}

#[derive(Default)]
pub struct UserForUpdateBuilder {
    update: UserForUpdate,
}

impl UserForUpdateBuilder {
    pub fn name(mut self, name: String) -> Self {
        self.update.name = Some(name);
        self
    }

    pub fn replit_id(mut self, replit_id: String) -> Self {
        self.update.replit_id = Some(replit_id);
        self
    }

    pub fn replit_profile_pic(mut self, replit_profile_pic: String) -> Self {
        self.update.replit_profile_pic = Some(replit_profile_pic);
        self
    }

    pub fn pubkey(mut self, pubkey: String) -> Self {
        self.update.pubkey = Some(pubkey);
        self
    }

    pub fn relays(mut self, relays: Vec<String>) -> Self {
        self.update.relays = Some(relays);
        self
    }

    pub fn federation_ids(mut self, federation_ids: Vec<String>) -> Self {
        self.update.federation_ids = Some(federation_ids);
        self
    }

    pub fn last_tweak(mut self, last_tweak: i64) -> Self {
        self.update.last_tweak = Some(last_tweak);
        self
    }

    pub fn connection_code_uuid(mut self, connection_code_uuid: String) -> Self {
        self.update.connection_code_uuid = Some(connection_code_uuid);
        self
    }

    pub fn build(self) -> UserForUpdate {
        self.update
    }
}
