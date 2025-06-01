use crate::{
    ServiceResult,
    services::{
        MapToUnknown, ServiceError,
        db::MaybeTxConn,
        id::{Identifier, UserID},
        kv::KVObject,
    },
};
use entity::{sea_orm_active_enums::Status, user_totp};
use expected_error::StatusCode;
use expected_error_derive::ExpectedError;
use sea_orm::ActiveModelTrait;
use sea_orm::Set;
use thiserror::Error;
use totp_rs::{Algorithm, Secret, TOTP};

use super::get_user_by_id;

const TOTP_ALGO: Algorithm = Algorithm::SHA1;
const TOTP_DIGITS: usize = 6;
const TOTP_STEP: u64 = 30;
const TOTP_SKEW: u8 = 1;

pub struct TotpSetup {
    pub qr_code_png_base64: String,
    pub url: String,
}

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum TotpError {
    #[error("TOTP setup failed: user not found")]
    #[ee(status(StatusCode::NOT_FOUND))]
    UserNotFound,
}

pub async fn setup_user_totp(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    user_id: UserID,
    my_domain: &str,
) -> ServiceResult<TotpSetup> {
    let user = get_user_by_id(conn, rconn, user_id).await?;
    let user = match user {
        Some(u) => u,
        None => return Err(ServiceError::known(TotpError::UserNotFound)),
    };

    let secret = Secret::generate_secret();

    let totp = TOTP::new(
        TOTP_ALGO,
        TOTP_DIGITS,
        TOTP_SKEW,
        TOTP_STEP,
        secret.to_bytes().unwrap(),
        Some(my_domain.to_owned()),
        user.id.to_string(),
    )
    .map_err_unknown()?;

    let user_totp = user_totp::ActiveModel {
        id: Set(user_id.as_db()),
        secret: Set(secret.to_string()),
        status: Set(Status::Setup),
        created_at: Set(chrono::Utc::now().naive_utc()),
    };
    user_totp.insert(conn).await.map_err_unknown()?;

    Ok(TotpSetup {
        qr_code_png_base64: totp.get_qr_base64().unwrap(),
        url: totp.get_url(),
    })
}
