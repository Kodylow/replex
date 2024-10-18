use anyhow::Result;
use config::CONFIG;
use multimint::MultiMint;

use crate::{config, db::Db};

#[derive(Clone)]
pub struct AppState {
    pub fm: MultiMint,
    pub db: Db,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let fm = MultiMint::new(CONFIG.fm_db_path.clone()).await?;
        let db = Db::new(CONFIG.pg_db.clone()).await?;
        db.setup_schema().await?;

        Ok(Self { fm, db })
    }
}
