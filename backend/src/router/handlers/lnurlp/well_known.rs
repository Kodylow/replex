use std::str::FromStr;

use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use multimint::fedimint_core::Amount;
use nostr_sdk::bitcoin::XOnlyPublicKey;
use serde::ser::{SerializeTuple, Serializer};
use serde::{Deserialize, Serialize};
use tracing::info;
use url::Url;

use super::{LnurlStatus, LnurlType};
use crate::config::CONFIG;
use crate::error::AppError;
use crate::state::AppState;

#[derive(Serialize, Deserialize, Debug)]
pub enum MetadataType {
    TextPlain,
    ImagePngBase64,
    ImageJpegBase64,
    TextEmail,
    TextIdentifier,
}

#[derive(Deserialize)]
pub struct MetadataEntry {
    pub metadata_type: MetadataType,
    pub content: String,
}

impl Serialize for MetadataEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&format!("{:?}", self.metadata_type))?;
        tup.serialize_element(&self.content)?;
        tup.end()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LnurlWellKnownResponse {
    pub callback: Url,
    pub max_sendable: Amount,
    pub min_sendable: Amount,
    pub metadata: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_allowed: Option<i32>,
    pub tag: LnurlType,
    pub status: LnurlStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nostr_pubkey: Option<XOnlyPublicKey>,
    pub allows_nostr: bool,
}

#[axum_macros::debug_handler]
pub async fn handle_well_known(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<LnurlWellKnownResponse>, AppError> {
    // see if username exists in nostr.json
    info!("well_known called with username: {}", username);
    match state.db.users().get_by_name(&username).await? {
        Some(user) => {
            let res = LnurlWellKnownResponse {
                callback: format!("https://{}/lnurlp/{}/callback", CONFIG.domain, username)
                    .parse()?,
                max_sendable: Amount { msats: 100000 },
                min_sendable: Amount { msats: 1000 },
                metadata: "".to_string(),
                comment_allowed: None,
                tag: LnurlType::PayRequest,
                status: LnurlStatus::Ok,
                nostr_pubkey: Some(XOnlyPublicKey::from_str(&user.pubkey)?),
                allows_nostr: true,
            };

            Ok(Json(res))
        }
        None => Err(AppError::new(
            StatusCode::NOT_FOUND,
            anyhow!("User not found"),
        )),
    }
}
