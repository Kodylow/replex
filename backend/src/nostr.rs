use anyhow::Result;

use multimint::fedimint_client::ClientHandleArc;
use nostr_sdk::FromBech32;
use nostr_sdk::SecretKey;
use nostr_sdk::ToBech32;
use tracing::info;

use crate::model::invoices::Invoice;
use crate::model::users::User;

#[derive(Clone)]
pub struct Nostr {
    pub client: nostr_sdk::Client,
}

impl Nostr {
    pub fn new(nsec: &str) -> Result<Self> {
        let secret_key = SecretKey::from_bech32(nsec)?;
        let keys = nostr_sdk::Keys::new(secret_key);
        info!("Nostr npub: {}", keys.public_key().to_bech32()?);
        info!("Nostr nsec: {}", keys.secret_key().to_bech32()?);
        let client = nostr_sdk::Client::new(keys.clone());
        Ok(Self { client })
    }

    pub async fn add_relays(&self, relays: &[String]) -> Result<()> {
        info!("Adding {} relays", relays.len());
        for relay in relays {
            self.client.add_relay(relay).await?;
        }
        Ok(())
    }

    pub async fn notify_user(&self, user: &User, invoice: Invoice) -> Result<()> {
        todo!()
    }
}
