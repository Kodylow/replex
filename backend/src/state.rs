use std::time::Duration;

use anyhow::Result;
use config::CONFIG;
use multimint::{
    fedimint_client::ClientHandleArc, fedimint_core::Amount,
    fedimint_mint_client::MintClientModule, MultiMint,
};

use crate::{
    config,
    model::{invoices::Invoice, users::User, Db},
    nostr::Nostr,
};

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

    pub async fn notify_user(
        client: &ClientHandleArc,
        user: &User,
        invoice: Invoice,
    ) -> Result<()> {
        let mint = client.get_first_module::<MintClientModule>();
        let (operation_id, notes) = mint
            .spend_notes(
                Amount::from_msats(invoice.amount as u64),
                Duration::from_secs(604800),
                false,
                (),
            )
            .await?;

        todo!()
    }
}
