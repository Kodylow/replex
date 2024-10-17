#![allow(dead_code)]
use anyhow::{anyhow, Result};
use serde::Serialize;
use sqlb::{Fields, HasFields};
use sqlx::FromRow;

use super::base::{self, DbBmc};
use super::ModelManager;

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct AppUser {
    pub id: i32,
    pub name: String,
    pub pubkey: String,
    pub relay: String,
    pub federation_id: String,
}

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct AppUserForCreate {
    pub name: String,
    pub pubkey: String,
    pub relay: String,
    pub federation_id: String,
}

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct AppUserForUpdate {
    pub pubkey: Option<String>,
    pub name: Option<String>,
    pub relay: Option<String>,
    pub federation_id: Option<String>,
}

pub struct AppUserBmc;

impl DbBmc for AppUserBmc {
    const TABLE: &'static str = "app_user";
}

impl AppUserBmc {
    pub async fn create(mm: &ModelManager, user_c: AppUserForCreate) -> Result<i32> {
        base::create::<Self, _>(mm, user_c).await
    }

    pub async fn get(mm: &ModelManager, id: i32) -> Result<AppUser> {
        base::get::<Self, _>(mm, id).await
    }

    pub async fn get_by_name(mm: &ModelManager, name: &str) -> Result<AppUser> {
        let user: AppUser = sqlb::select()
            .table(Self::TABLE)
            .columns(AppUser::field_names())
            .and_where("name", "=", name)
            .fetch_optional(mm.db())
            .await?
            .ok_or(anyhow!(
                "User not found in table '{}', {}: {}",
                Self::TABLE,
                "name",
                name
            ))?;

        Ok(user)
    }

    pub async fn list(mm: &ModelManager) -> Result<Vec<AppUser>> {
        base::list::<Self, _>(mm).await
    }

    pub async fn update(mm: &ModelManager, id: i32, user_u: AppUserForUpdate) -> Result<()> {
        base::update::<Self, _>(mm, id, user_u).await
    }

    pub async fn delete(mm: &ModelManager, id: i32) -> Result<()> {
        base::delete::<Self>(mm, id).await
    }
}
