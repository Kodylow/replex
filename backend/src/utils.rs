use std::fmt::Display;
use std::str::FromStr;

use crate::error::AppError;
use crate::model::app_user::{AppUser, AppUserForUpdate};
use crate::model::invoice::Invoice;
use crate::router::handlers::lnurlp::callback::spawn_invoice_subscription;
use crate::state::AppState;
use anyhow::Result;
use axum::http::StatusCode;
use multimint::fedimint_client::ClientHandleArc;
use multimint::{fedimint_core::config::FederationId, fedimint_ln_client::LightningClientModule};
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use tokio::fs;
use tracing::error;

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
    let json_content = fs::read_to_string("nostr.json").await?;
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
        let sql = "SELECT * FROM app_users WHERE name = $1";
        if let Some(user) = state
            .db
            .query_opt::<AppUser>(sql, &[&name.to_string()])
            .await?
        {
            let app_user = AppUserForUpdate::builder()
                .name(name.to_string())
                .pubkey(pubkey.to_string())
                .relays(user_relays)
                .federation_ids(user_federation_ids)
                .build();
            let sql = "UPDATE app_users SET name = $1, pubkey = $2, relays = $3, federation_ids = $4 WHERE id = $5";
            state
                .db
                .execute(
                    sql,
                    &[
                        &app_user.name,
                        &app_user.pubkey,
                        &app_user.relays,
                        &app_user.federation_ids,
                        &user.id,
                    ],
                )
                .await?;
        } else {
            let app_user = AppUser::builder()
                .name(name.clone())
                .pubkey(pubkey.to_string())
                .relays(user_relays)
                .federation_ids(user_federation_ids)
                .last_tweak(0)
                .build()?;
            let sql = "INSERT INTO app_users (name, pubkey, relays, federation_ids, last_tweak) VALUES ($1, $2, $3, $4, $5)";
            state
                .db
                .execute(
                    sql,
                    &[
                        &app_user.name,
                        &app_user.pubkey,
                        &app_user.relays,
                        &app_user.federation_ids,
                        &app_user.last_tweak,
                    ],
                )
                .await?;
        }
    }

    Ok(())
}

/// Starts subscription for all pending invoices from previous run
pub async fn handle_pending_invoices(state: AppState) -> Result<()> {
    // Group invoices by federation_id
    let sql = "SELECT * FROM invoices WHERE state = 'pending' GROUP BY federation_id";
    let invoices_by_federation = state.db.query_group_by::<Invoice>(sql, &[]).await?;

    for (federation_id, invoices) in invoices_by_federation {
        // Get the corresponding multimint client for the federation_id
        match FederationId::from_str(&federation_id) {
            Ok(federation_id) => {
                if let Some(client) = state.fm.clients.lock().await.get(&federation_id) {
                    let ln = client.get_first_module::<LightningClientModule>();
                    for invoice in invoices {
                        // Create subscription to operation if it exists
                        if let Ok(subscription) = ln
                            .subscribe_ln_receive(invoice.op_id.parse().expect("invalid op_id"))
                            .await
                        {
                            spawn_invoice_subscription(
                                state.clone(),
                                invoice.clone(),
                                subscription,
                            )
                            .await?;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Invalid federation_id: {}", e);
            }
        }
    }

    Ok(())
}

pub async fn get_federation_and_client(
    state: &AppState,
    user: &AppUser,
) -> Result<(FederationId, ClientHandleArc), AppError> {
    let federation_id = FederationId::from_str(&user.federation_ids[0]).map_err(|e| {
        AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!("Invalid federation_id for user {}: {}", user.name, e),
        )
    })?;

    let locked_clients = state.fm.clients.lock().await.clone();
    let client = locked_clients.get(&federation_id).ok_or_else(|| {
        AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!("FederationId not found in multimint map"),
        )
    })?;

    Ok((federation_id, client.clone()))
}
