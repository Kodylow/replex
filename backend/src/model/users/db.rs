use crate::model::users::{User, UserForCreate, UserForUpdate};
use crate::model::Db;
use anyhow::Result;

pub struct UserDb(pub Db);

impl UserDb {
    pub async fn create(&self, user: UserForCreate) -> Result<User> {
        let sql = "INSERT INTO users (name, pubkey, last_tweak, relays, federation_ids) VALUES ($1, $2, $3, $4, $5) RETURNING *";
        self.0
            .query_one::<User>(
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

    pub async fn get(&self, id: i32) -> Result<Option<User>> {
        let sql = "SELECT * FROM users WHERE id = $1";
        self.0.query_opt::<User>(sql, &[&id]).await
    }

    pub async fn get_by_name(&self, username: &str) -> Result<Option<User>> {
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
}

pub async fn get_user(db: &Db, username: &str) -> Result<User> {
    let sql = "SELECT * FROM users WHERE name = $1";
    db.query_one::<User>(sql, &[&username])
        .await
        .map_err(|e| e.into())
}
