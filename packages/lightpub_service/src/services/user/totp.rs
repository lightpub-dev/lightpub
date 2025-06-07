use crate::{
    ServiceResult,
    services::{
        MapToUnknown, ServiceError,
        db::{Conn, MaybeTxConn},
        id::{Identifier, UserID},
        kv::KVObject,
    },
};
use entity::{sea_orm_active_enums::Status, user_totp, user_totp_backup};
use expected_error::StatusCode;
use expected_error_derive::ExpectedError;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelTrait, SqlErr};
use sea_orm::{EntityTrait, QueryFilter, Set};
use thiserror::Error;
use totp_rs::{Algorithm, Secret, TOTP};

use super::get_user_by_id;

const TOTP_ALGO: Algorithm = Algorithm::SHA1;
const TOTP_DIGITS: usize = 6;
const TOTP_STEP: u64 = 30;
const TOTP_SKEW: u8 = 1;

#[derive(Debug, Clone)]
pub enum TotpSetup {
    Success {
        qr_code_png_base64: String,
        url: String,
    },
    AlreadySetup,
}

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum TotpError {
    #[error("TOTP setup failed: user not found")]
    #[ee(status(StatusCode::NOT_FOUND))]
    UserNotFound,
}

pub async fn setup_user_totp(
    conn: &Conn,
    rconn: &KVObject,
    user_id: UserID,
    force: bool,
    my_domain: &str,
) -> ServiceResult<TotpSetup> {
    let tx = conn.as_tx().await?.into();

    let user = get_user_by_id(&tx, rconn, user_id).await?;
    let user = match user {
        Some(u) => u,
        None => return Err(ServiceError::known(TotpError::UserNotFound)),
    };

    let secret = Secret::generate_secret().to_encoded();

    if force {
        user_totp::Entity::delete_many()
            .filter(user_totp::Column::Id.eq(user_id.as_db()))
            .exec(&tx)
            .await
            .map_err_unknown()?;
        user_totp_backup::Entity::delete_many()
            .filter(user_totp_backup::Column::UserId.eq(user_id.as_db()))
            .exec(&tx)
            .await
            .map_err_unknown()?;
    }

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
    match user_totp.insert(&tx).await {
        Ok(_) => {}
        Err(e) => match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                return Ok(TotpSetup::AlreadySetup);
            }
            _ => {
                return Err(ServiceError::unknown(e));
            }
        },
    }

    tx.commit().await?;

    Ok(TotpSetup::Success {
        qr_code_png_base64: totp.get_qr_base64().unwrap(),
        url: totp.get_url(),
    })
}

pub async fn deactivate_user_totp(
    conn: &Conn,
    rconn: &KVObject,
    user_id: UserID,
) -> ServiceResult<()> {
    let tx = conn.as_tx().await?.into();

    let user = get_user_by_id(&tx, rconn, user_id).await?;
    let user = match user {
        Some(u) => u,
        None => return Err(ServiceError::known(TotpError::UserNotFound)),
    };

    user_totp::Entity::delete_many()
        .filter(user_totp::Column::Id.eq(user_id.as_db()))
        .exec(&tx)
        .await
        .map_err_unknown()?;
    user_totp_backup::Entity::delete_many()
        .filter(user_totp_backup::Column::UserId.eq(user_id.as_db()))
        .exec(&tx)
        .await
        .map_err_unknown()?;

    tx.commit().await?;

    Ok(())
}

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum TotpVerifyResultError {
    #[error("Bad TOTP code")]
    #[ee(status(StatusCode::UNAUTHORIZED))]
    BadCode,
    #[error("Trying to verify against TOTP that is pending setup")]
    #[ee(status(StatusCode::UNAUTHORIZED))]
    PendingSetup,
    #[error("TOTP is not set up for this user")]
    #[ee(status(StatusCode::UNAUTHORIZED))]
    NotSetup,
}

#[derive(Debug, Clone)]
pub enum TotpVerifyResult {
    Success,
    Error(TotpVerifyResultError),
}

pub async fn user_totp_verify(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    user_id: UserID,
    allow_pending: bool,
    code: &str,
    my_domain: &str,
) -> ServiceResult<TotpVerifyResult> {
    let user = get_user_by_id(conn, rconn, user_id).await?;
    let user = match user {
        Some(u) => u,
        None => return Err(ServiceError::known(TotpError::UserNotFound)),
    };

    let totp_entry = user_totp::Entity::find()
        .filter(user_totp::Column::Id.eq(user_id.as_db()))
        .one(conn)
        .await
        .map_err_unknown()?;
    let totp_entry = match totp_entry {
        Some(entry) => entry,
        None => {
            return Ok(TotpVerifyResult::Error(TotpVerifyResultError::NotSetup));
        }
    };

    if !allow_pending && totp_entry.status == Status::Setup {
        return Ok(TotpVerifyResult::Error(TotpVerifyResultError::PendingSetup));
    }

    let secret = Secret::Encoded(totp_entry.secret.clone());

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

    if !totp.check(code, chrono::Utc::now().timestamp() as u64) {
        return Ok(TotpVerifyResult::Error(TotpVerifyResultError::BadCode));
    }

    return Ok(TotpVerifyResult::Success);
}
