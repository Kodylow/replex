use crate::model::users::{User, UserForCreate, UserForUpdate};
use crate::model::Db;
use anyhow::Result;
use tracing::info;

pub struct UserDb(pub Db);

impl UserDb {
    pub async fn create(&self, user: UserForCreate) -> Result<User> {
        let sql = "INSERT INTO users (name, replit_id, replit_profile_pic, pubkey, relays, federation_ids, connection_code_uuid, last_tweak) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *";
        self.0
            .query_one::<User>(
                sql,
                &[
                    &user.name,
                    &user.replit_id,
                    &user.replit_profile_pic,
                    &user.pubkey,
                    &user.relays,
                    &user.federation_ids,
                    &user.connection_code_uuid,
                    &user.last_tweak,
                ],
            )
            .await
    }

    pub async fn get(&self, id: i32) -> Result<Option<User>> {
        let sql = "SELECT * FROM users WHERE id = $1";
        self.0.query_opt::<User>(sql, &[&id]).await
    }

    pub async fn get_by_name(&self, username: &str) -> Result<Option<User>> {
        info!("Getting user by name: {}", username);
        let sql = "SELECT * FROM users WHERE name = $1";
        self.0.query_opt::<User>(sql, &[&username]).await
    }

    pub async fn update(&self, id: i32, user: UserForUpdate) -> Result<User> {
        let mut updates = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        let mut param_count = 1;

        if let Some(name) = &user.name {
            updates.push(format!("name = ${}", param_count));
            params.push(name);
            param_count += 1;
        }
        if let Some(replit_id) = &user.replit_id {
            updates.push(format!("replit_id = ${}", param_count));
            params.push(replit_id);
            param_count += 1;
        }
        if let Some(replit_profile_pic) = &user.replit_profile_pic {
            updates.push(format!("replit_profile_pic = ${}", param_count));
            params.push(replit_profile_pic);
            param_count += 1;
        }
        if let Some(pubkey) = &user.pubkey {
            updates.push(format!("pubkey = ${}", param_count));
            params.push(pubkey);
            param_count += 1;
        }
        if let Some(relays) = &user.relays {
            updates.push(format!("relays = ${}", param_count));
            params.push(relays);
            param_count += 1;
        }
        if let Some(federation_ids) = &user.federation_ids {
            updates.push(format!("federation_ids = ${}", param_count));
            params.push(federation_ids);
            param_count += 1;
        }
        if let Some(connection_code_uuid) = &user.connection_code_uuid {
            updates.push(format!("connection_code_uuid = ${}", param_count));
            params.push(connection_code_uuid);
            param_count += 1;
        }
        if let Some(last_tweak) = &user.last_tweak {
            updates.push(format!("last_tweak = ${}", param_count));
            params.push(last_tweak);
            param_count += 1;
        }

        let sql = format!(
            "UPDATE users SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_count
        );
        params.push(&id);

        self.0.query_one::<User>(&sql, &params).await
    }

    pub async fn update_tweak(&self, user: &User, tweak: i64) -> Result<()> {
        let sql = "UPDATE users SET last_tweak = $1 WHERE id = $2";
        self.0.execute(sql, &[&tweak, &user.id]).await?;
        Ok(())
    }

    pub async fn update_or_create_user(
        &self,
        name: &str,
        replit_id: &str,
        replit_profile_pic: &str,
        pubkey: &str,
        relays: Vec<String>,
        federation_ids: Vec<String>,
    ) -> Result<()> {
        if let Some(user) = self.get_by_name(name).await? {
            info!("User {} already exists", name);
            let user_for_update = UserForUpdate::builder()
                .name(name.to_string())
                .replit_id(replit_id.to_string())
                .replit_profile_pic(replit_profile_pic.to_string())
                .pubkey(pubkey.to_string())
                .relays(relays)
                .federation_ids(federation_ids)
                .build();
            self.update(user.id, user_for_update).await?;
        } else {
            info!("User {} does not exist", name);
            let user = UserForCreate::new(
                name.to_string(),
                replit_id.to_string(),
                replit_profile_pic.to_string(),
                pubkey.to_string(),
                relays,
                federation_ids,
            );
            self.create(user).await?;
        }

        Ok(())
    }

    pub async fn create_new_user(
        &self,
        replit_user_id: &str,
        replit_user_name: &str,
        replit_profile_pic: &str,
        pubkey: &str,
        relays: Vec<String>,
        federation_ids: Vec<String>,
    ) -> Result<User> {
        let user = UserForCreate::new(
            replit_user_name.to_string(),
            replit_user_id.to_string(),
            replit_profile_pic.to_string(),
            pubkey.to_string(),
            relays,
            federation_ids,
        );

        self.create(user).await
    }
}
