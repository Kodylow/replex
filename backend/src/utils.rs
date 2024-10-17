use std::fmt::Display;
use std::str::FromStr;

use crate::model::app_user::AppUserForUpdate;
use crate::state::AppState;
use crate::{
    model::{
        app_user::{AppUserBmc, AppUserForCreate},
        invoice::InvoiceBmc,
    },
    router::handlers::lnurlp::callback::spawn_invoice_subscription,
};
use anyhow::Result;
use itertools::Itertools;
use multimint::{fedimint_core::config::FederationId, fedimint_ln_client::LightningClientModule};
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

/// Loads users and keys from nostr.json
pub async fn load_users_and_keys(state: AppState) -> Result<()> {
    let json_content = fs::read_to_string("backend/nostr.json")?;
    let json: Value = serde_json::from_str(&json_content)?;

    let names = json["names"]
        .as_object()
        .ok_or(anyhow::anyhow!("Invalid 'names' structure"))?;
    let relays = json["relays"]
        .as_object()
        .ok_or(anyhow::anyhow!("Invalid 'relays' structure"))?;
    let federation_ids = json["federation_ids"]
        .as_object()
        .ok_or(anyhow::anyhow!("Invalid 'federation_ids' structure"))?;

    for (name, pubkey) in names {
        let pubkey = pubkey.as_str().ok_or(anyhow::anyhow!("Invalid pubkey"))?;
        let user_relays = relays[pubkey]
            .as_array()
            .ok_or(anyhow::anyhow!("Invalid relays for user"))?
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect::<Vec<String>>();
        let user_federation_ids = federation_ids[pubkey]
            .as_array()
            .ok_or(anyhow::anyhow!("Invalid federation_ids for user"))?
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect::<Vec<String>>();

        // Update user if it already exists, keep last_tweak
        if let Ok(user) = AppUserBmc::get_by_name(&state.mm, name.as_str()).await {
            let app_user = AppUserForUpdate::new()
                .set_pubkey(Some(pubkey.to_string()))
                .set_name(Some(name.to_string()))
                .set_relay(Some(user_relays[0].clone()))
                .set_federation_id(Some(user_federation_ids[0].clone()));
            AppUserBmc::update(&state.mm, user.id, app_user).await?;
        // Create user if it doesn't exist
        } else {
            let app_user = AppUserForCreate {
                name: name.clone(),
                pubkey: pubkey.to_string(),
                relay: user_relays[0].clone(),
                federation_id: user_federation_ids[0].clone(),
                last_tweak: 0, // Set to 0 for new users
            };
            AppUserBmc::create(&state.mm, app_user).await?;
        }
    }

    Ok(())
}

/// Starts subscription for all pending invoices from previous run
pub async fn handle_pending_invoices(state: AppState) -> Result<()> {
    let invoices = InvoiceBmc::get_pending(&state.mm).await?;

    // Group invoices by federation_id
    let invoices_by_federation = invoices
        .into_iter()
        .chunk_by(|i| i.federation_id.clone())
        .into_iter()
        .map(|(federation_id, invs)| (federation_id, invs.collect::<Vec<_>>()))
        .collect::<HashMap<_, _>>();

    for (federation_id, invoices) in invoices_by_federation {
        // Get the corresponding multimint client for the federation_id
        if let Ok(federation_id) = FederationId::from_str(&federation_id) {
            if let Some(client) = state.fm.clients.lock().await.get(&federation_id) {
                let ln = client.get_first_module::<LightningClientModule>();
                for invoice in invoices {
                    // Create subscription to operation if it exists
                    if let Ok(subscription) = ln
                        .subscribe_ln_receive(invoice.op_id.parse().expect("invalid op_id"))
                        .await
                    {
                        let user = AppUserBmc::get(&state.mm, invoice.app_user_id).await?;
                        spawn_invoice_subscription(
                            state.clone(),
                            invoice.id,
                            user.clone(),
                            subscription,
                        )
                        .await;
                    }
                }
            }
        }
    }

    Ok(())
}
