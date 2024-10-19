use crate::config::CONFIG;
use crate::error::AppError;
use crate::model::invoices::Invoice;
use crate::model::users::User;
use crate::model::Db;
use crate::router::handlers::lnurlp::callback::{LnurlCallbackParams, LnurlCallbackResponse};
use crate::router::handlers::lnurlp::LnurlStatus;
use anyhow::Result;
use axum::http::StatusCode;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::core::OperationId;
use multimint::fedimint_core::secp256k1::PublicKey;
use multimint::fedimint_core::Amount;
use multimint::fedimint_ln_client::LightningClientModule;
use multimint::fedimint_ln_common::lightning_invoice::{
    Bolt11Invoice, Bolt11InvoiceDescription, Description,
};
use multimint::fedimint_mint_client::MintClientModule;
use std::str::FromStr;
use std::time::Duration;
use url::Url;

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
        PublicKey::from_str(&user.pubkey)?,
        tweak as u64,
        (),
        None,
    )
    .await
    .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, anyhow::anyhow!(e)))
}

pub fn create_verify_url(username: &str, op_id: &OperationId) -> Result<Url> {
    Url::parse(&format!(
        "http://{}:{}/lnurlp/{}/verify/{}",
        CONFIG.domain,
        CONFIG.port,
        username,
        op_id.fmt_full().to_string()
    ))
    .map_err(|e| anyhow::anyhow!(e))
}

pub fn create_callback_response(
    bolt11: String,
    verify_url: Url,
) -> Result<LnurlCallbackResponse, AppError> {
    Ok(LnurlCallbackResponse {
        pr: bolt11,
        success_action: None,
        status: LnurlStatus::Ok,
        reason: None,
        verify: verify_url,
        routes: Some(vec![]),
    })
}

pub async fn notify_user(client: &ClientHandleArc, db: &Db, invoice: Invoice) -> Result<()> {
    let user = db.users().get(invoice.user_id).await?;
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
