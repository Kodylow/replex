use anyhow::Result;
use config::CONFIG;
use multimint::MultiMint;

use crate::{config, model::Db, nostr::Nostr};

#[derive(Clone)]
pub struct AppState {
    pub mm: MultiMint,
    pub db: Db,
    pub nostr: Nostr,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let mm = MultiMint::new(CONFIG.fm_db_path.clone()).await?;
        let db = Db::new(CONFIG.pg_db.clone()).await?;
        let nostr = Nostr::new(&CONFIG.nostr_nsec)?;
        nostr.add_relays(&CONFIG.nostr_relays).await?;
        db.setup_schema().await?;

        Ok(Self { mm, db, nostr })
    }
}
