use std::str::FromStr;

use crate::error::AppError;
use crate::model::users::User;
use crate::router::handlers::lnurlp::callback::LnurlCallbackParams;
use anyhow::Result;
use axum::http::StatusCode;
use multimint::fedimint_core::core::OperationId;
use multimint::fedimint_core::secp256k1::PublicKey;
use multimint::fedimint_core::Amount;
use multimint::fedimint_ln_client::LightningClientModule;
use multimint::fedimint_ln_common::lightning_invoice::{
    Bolt11Invoice, Bolt11InvoiceDescription, Description,
};
use nostr_sdk::bitcoin::XOnlyPublicKey;
use nostr_sdk::secp256k1::Parity;

pub const MIN_AMOUNT: u64 = 1000;

pub fn validate_amount(amount: u64) -> Result<(), AppError> {
    if amount < MIN_AMOUNT {
        return Err(AppError {
            error: anyhow::anyhow!("Amount < MIN_AMOUNT"),
            status: StatusCode::BAD_REQUEST,
        });
    }
    Ok(())
}

pub async fn create_invoice(
    ln: &LightningClientModule,
    params: &LnurlCallbackParams,
    user: &User,
    tweak: i64,
) -> Result<(OperationId, Bolt11Invoice, [u8; 32]), AppError> {
    let xonly_pubkey = XOnlyPublicKey::from_str(&user.pubkey)?;
    let pubkey = PublicKey::from_str(&xonly_pubkey.public_key(Parity::Even).to_string())?;
    ln.create_bolt11_invoice_for_user_tweaked(
        Amount {
            msats: params.amount,
        },
        Bolt11InvoiceDescription::Direct(&Description::new(
            params
                .comment
                .clone()
                .unwrap_or("hermes address payment".to_string()),
        )?),
        None,
        pubkey,
        tweak as u64,
        (),
        None,
    )
    .await
    .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, anyhow::anyhow!(e)))
}
