use totp_rs::{Algorithm, Secret, TOTP};

use crate::{
    ServiceResult,
    services::{db::MaybeTxConn, id::UserID, kv::KVObject},
};

use super::get_user_by_id;

const TOTP_ALGO: Algorithm = Algorithm::SHA1;
const TOTP_DIGITS: usize = 6;
const TOTP_STEP: u64 = 30;
const TOTP_SKEW: u8 = 1;

pub struct TotpSetup {
    pub qr_code_png_base64: String,
    pub url: String,
}

pub async fn setup_user_totp(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    user_id: UserID,
    my_domain: &str,
) -> ServiceResult<TotpSetup> {
    let user = get_user_by_id(conn, rconn, user_id).await?;

    let totp = TOTP::new(
        TOTP_ALGO,
        TOTP_DIGITS,
        TOTP_SKEW,
        TOTP_STEP,
        Secret::generate_secret(),
        Some(my_domain.to_owned()),
    );

    todo!()
}
