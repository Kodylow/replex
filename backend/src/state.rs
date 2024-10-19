use std::str::FromStr;

use anyhow::Result;
use config::CONFIG;
use multimint::MultiMint;
use nostr_sdk::bip39::Mnemonic;

use crate::{config, model::Db, nostr::Nostr};

// Tweaks the root mnemonic to be used specifically for Nostr
const NOSTR_PASSPHRASE: &str = "nostr";

#[derive(Clone)]
pub struct AppState {
    pub fm: MultiMint,
    pub db: Db,
    pub nostr: Nostr,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let fm = MultiMint::new(CONFIG.fm_db_path.clone()).await?;
        let db = Db::new(CONFIG.pg_db.clone()).await?;
        let nostr_sk_bytes = Mnemonic::from_str(&CONFIG.mnemonic)?.to_seed(NOSTR_PASSPHRASE);
        let nostr = Nostr::new(&nostr_sk_bytes)?;
        nostr.add_relays(&CONFIG.nostr_relays).await?;
        db.setup_schema().await?;

        Ok(Self { fm, db, nostr })
    }
}
