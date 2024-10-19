use crate::model::users::{UserForCreate, UserForUpdate};
use crate::state::AppState;
use anyhow::Result;
use serde_json::Value;
use tokio::fs;
use tracing::info;

pub async fn load_users_and_keys(state: AppState) -> Result<()> {
    info!("Loading users and keys from nostr.json");
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

        update_or_create_user(&state, name, pubkey, user_relays, user_federation_ids).await?;
    }

    Ok(())
}

async fn update_or_create_user(
    state: &AppState,
    name: &str,
    pubkey: &str,
    relays: Vec<String>,
    federation_ids: Vec<String>,
) -> Result<()> {
    let user_db = state.db.users();

    if let Some(user) = user_db.get_by_name(name).await? {
        info!("User {} already exists", name);
        let user_for_update = UserForUpdate::builder()
            .name(name.to_string())
            .pubkey(pubkey.to_string())
            .relays(relays)
            .federation_ids(federation_ids)
            .build();
        user_db.update(user.id, user_for_update).await?;
    } else {
        info!("User {} does not exist", name);
        let user = UserForCreate::builder()
            .name(name.to_string())
            .pubkey(pubkey.to_string())
            .relays(relays)
            .federation_ids(federation_ids)
            .last_tweak(0)
            .build()?;
        user_db.create(user).await?;
    }

    Ok(())
}
