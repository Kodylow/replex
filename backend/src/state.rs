use anyhow::Result;
use config::CONFIG;
use multimint::MultiMint;

use crate::config;
use crate::model::ModelManager;

#[derive(Clone)]
pub struct AppState {
    pub fm: MultiMint,
    pub mm: ModelManager,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let fm = MultiMint::new(CONFIG.fm_db_path.clone()).await?;
        let mm = ModelManager::new().await?;

        Ok(Self { fm, mm })
    }
}
