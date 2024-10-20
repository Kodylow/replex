use std::str::FromStr;

use anyhow::Result;

use nostr_sdk::FromBech32;
use nostr_sdk::PublicKey;
use nostr_sdk::SecretKey;
use nostr_sdk::ToBech32;
use serde_json::json;
use tracing::info;

use crate::model::invoices::Invoice;

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

    pub async fn notify_user_invoice_settled(&self, invoice: Invoice) -> Result<()> {
        let dm = self
            .client
            .send_private_msg(
                PublicKey::from_str(&invoice.user_pubkey)?,
                json!(invoice).to_string(),
                None,
            )
            .await?;

        info!("Sent nostr dm: {:?}", dm);
        Ok(())
    }
}
