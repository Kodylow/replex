use anyhow::Result;

use nostr_sdk::SecretKey;
use nostr_sdk::ToBech32;
use tracing::info;

#[derive(Clone)]
pub struct Nostr {
    pub client: nostr_sdk::Client,
}

impl Nostr {
    pub fn new(secret_key_bytes: &[u8]) -> Result<Self> {
        let secret_key = SecretKey::from_slice(secret_key_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid secret key"))?;
        let keys = nostr_sdk::Keys::new(secret_key);
        info!("Nostr npub: {}", keys.public_key().to_bech32()?);
        info!("Nostr nsec: {}", keys.secret_key().to_bech32()?);
        let client = nostr_sdk::Client::new(keys.clone());
        Ok(Self { client })
    }

    pub async fn add_relays(&self, relays: &[String]) -> Result<()> {
        for relay in relays {
            self.client.add_relay(relay).await?;
        }
        Ok(())
    }
}
