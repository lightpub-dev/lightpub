/*
Lightpub: a simple ActivityPub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

/// Route handlers for /auth
use std::{borrow::Cow, env::VarError};

use actix_session::Session;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::{header::HeaderMap, StatusCode},
    middleware::Next,
    post, web, HttpMessage, Responder,
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use validator::Validate;

use lightpub_service::services::{
    auth::{
        change_password, check_password_user, check_user_login_expiration, logout_all,
        register_user,
    },
    create_error_simple, create_error_simple_err,
    db::Conn,
    id::{Identifier, UserID},
    MapToUnknown, ServiceError, ServiceResult,
};

use crate::AppState;

use super::{APIResponse, APIResponseBuilder};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    username: String,
    #[validate(length(min = 6))]
    password: String,
    nickname: String,
}
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    user_id: String,
}

#[post("/register")]
pub async fn api_register_user(
    st: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> ServiceResult<APIResponse<RegisterResponse>> {
    if !st.is_registration_open() {
        return create_error_simple(StatusCode::BAD_REQUEST, "registration is closed");
    }

    req.validate().map_err(ServiceError::validation)?;

    let result = register_user(st.conn(), &req.username, &req.nickname, &req.password).await?;

    Ok(APIResponseBuilder::default()
        .data(RegisterResponse {
            user_id: result.user_id.to_string(),
        })
        .redirect_to("/client/login")
        .build()
        .unwrap())
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    user_id: String,
    token: String,
}

const TOKEN_SESSION_KEY: &str = "token";

#[post("/login")]
pub async fn api_login_user(
    st: web::Data<AppState>,
    session: Session,
    req: web::Json<LoginRequest>,
) -> ServiceResult<APIResponse<LoginResponse>> {
    let user = check_password_user(st.conn(), &req.username, &req.password).await?;

    match user {
        None => {
            session.remove(TOKEN_SESSION_KEY);
            create_error_simple(StatusCode::UNAUTHORIZED, "bad login")
        }
        Some(user) => {
            let token = generate_jwt(
                user.user_id.to_string().as_str(),
                &get_jwt_secret_key().await?,
            )?;
            session
                .insert(TOKEN_SESSION_KEY, token.clone())
                .map_err_unknown()?;
            Ok(APIResponseBuilder::default()
                .data(LoginResponse {
                    user_id: user.user_id.to_string(),
                    token,
                })
                .redirect_to("/client/timeline")
                .build()
                .unwrap())
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogoutQuery {
    all: Option<bool>,
}
use actix_web::middleware::from_fn;

#[post("/logout", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_logout_user(
    st: web::Data<AppState>,
    session: Session,
    query: web::Query<LogoutQuery>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<APIResponse<()>> {
    session.remove(TOKEN_SESSION_KEY);

    if query.all.unwrap_or(false) {
        match auth.user_id() {
            Some(user_id) => logout_all(st.conn(), user_id).await?,
            None => {
                return create_error_simple(StatusCode::UNAUTHORIZED, "not logged in");
            }
        }
    }

    Ok(APIResponseBuilder::default()
        .data(())
        .redirect_to("/client/login")
        .build()
        .unwrap())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    new_password: String,
}

#[post("/changePassword", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_change_password(
    st: web::Data<AppState>,
    session: Session,
    auth: web::ReqData<AuthedUser>,
    req: web::Json<ChangePasswordRequest>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();
    let new_password = &req.new_password;

    change_password(st.conn(), user_id, new_password, true).await?;

    session.remove(TOKEN_SESSION_KEY);

    Ok(APIResponseBuilder::default()
        .data(())
        .redirect_to("/client/login")
        .build()
        .unwrap())
}

async fn get_cookie_from_req<'a>(
    headers: &'a HeaderMap,
    session: &'a Session,
) -> ServiceResult<Option<Cow<'a, str>>> {
    // check authorization header
    if let Some(bearer_token) = headers.get("Authorization") {
        let bearer_token = bearer_token.to_str().map_err_unknown()?;
        if bearer_token.starts_with("Bearer ") {
            let token = &bearer_token[7..];
            return Ok(Some(Cow::Borrowed(token)));
        }
    }

    // check cookie
    if let Some(token) = session.get::<String>(TOKEN_SESSION_KEY).map_err_unknown()? {
        return Ok(Some(Cow::Owned(token)));
    }

    Ok(None)
}

async fn check_auth_expired(conn: &Conn, user_id: &UserID, claims: &Claims) -> ServiceResult<()> {
    let logged_in_at = DateTime::<Utc>::from_timestamp(claims.iat as i64, 0).expect("bad iat");
    let ok = check_user_login_expiration(conn, user_id, logged_in_at).await?;
    match ok {
        None => create_error_simple(StatusCode::UNAUTHORIZED, "user not found"),
        Some(false) => create_error_simple(StatusCode::UNAUTHORIZED, "login expired"),
        _ => Ok(()),
    }
}

pub async fn middleware_auth_jwt_required(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let headers = req.headers().clone();
    let session = req.extract::<Session>().await.unwrap();
    let st = req.extract::<web::Data<AppState>>().await.unwrap();

    let token = get_cookie_from_req(&headers, &session).await?;
    match token {
        None => {
            return Err(create_error_simple_err(StatusCode::UNAUTHORIZED, "no token").into());
        }
        Some(token) => {
            let token = &token;
            let secret_key = get_jwt_decoding_key().await?;
            let claims = verify_jwt(token, &secret_key).await?;
            let user_id = UserID::from_string(claims.sub.as_str())
                .ok_or_else(|| create_error_simple_err(StatusCode::UNAUTHORIZED, "bad token"))?;
            check_auth_expired(st.conn(), &user_id, &claims).await?;
            req.extensions_mut().insert(AuthedUser {
                user_id: Some(user_id),
            });
            next.call(req).await
        }
    }
}

pub async fn middleware_auth_jwt_optional(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let headers = req.headers().clone();
    let session = req.extract::<Session>().await.unwrap();
    let st = req.extract::<web::Data<AppState>>().await.unwrap();

    let token = get_cookie_from_req(&headers, &session).await?;
    match token {
        None => {
            req.extensions_mut().insert(AuthedUser { user_id: None });
            next.call(req).await
        }
        Some(token) => {
            let token = &token;
            let secret_key = get_jwt_decoding_key().await?;
            let claims = verify_jwt(token, &secret_key).await?;

            let user_id = UserID::from_string(claims.sub.as_str())
                .ok_or_else(|| create_error_simple_err(StatusCode::UNAUTHORIZED, "bad token"))?;
            check_auth_expired(st.conn(), &user_id, &claims).await?;
            req.extensions_mut().insert(AuthedUser {
                user_id: Some(user_id),
            });
            next.call(req).await
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthedUser {
    user_id: Option<UserID>,
}

impl AuthedUser {
    pub fn is_authed(&self) -> bool {
        self.user_id.is_some()
    }

    /// 認証されたユーザーの ID を取得する。
    /// 認証していない場合は None を返す。
    pub fn user_id(&self) -> Option<UserID> {
        self.user_id.clone()
    }

    /// 認証されたユーザーの ID を取得する。
    ///
    /// # Panics
    /// 認証されていない場合は Panic する。
    /// middleware_auth_jwt_required を通した後は Panic しない。
    pub fn user_id_unwrap(&self) -> UserID {
        self.user_id.clone().unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    // aud: String,         // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    nbf: usize, // Optional. Not Before (as UTC timestamp)
    sub: String, // Optional. Subject (whom token refers to)
}

fn generate_jwt(user_id: &str, secret_key: &EncodingKey) -> ServiceResult<String> {
    let header = jsonwebtoken::Header::new(Algorithm::RS256);

    let iat = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let exp_duration = std::time::Duration::from_secs(60 * 60 * 24);
    let exp = iat + exp_duration.as_secs() as usize;
    let claims = Claims {
        exp,
        iat,
        iss: "lightpub".to_string(),
        nbf: iat,
        sub: user_id.to_string(),
    };

    jsonwebtoken::encode(&header, &claims, secret_key).map_err_unknown()
}

async fn verify_jwt(token: &str, secret_key: &DecodingKey) -> ServiceResult<Claims> {
    let result = jsonwebtoken::decode::<Claims>(
        token,
        secret_key,
        &jsonwebtoken::Validation::new(Algorithm::RS256),
    )
    .map(|data| data.claims);

    match result {
        Ok(c) => Ok(c),
        Err(_) => {
            return create_error_simple(StatusCode::UNAUTHORIZED, "bad token");
        }
    }
}

async fn get_jwt_secret_key() -> ServiceResult<EncodingKey> {
    let key = read_jwt_private_key_().await?;
    Ok(EncodingKey::from_rsa_pem(key.as_bytes()).map_err_unknown()?)
}

async fn get_jwt_decoding_key() -> ServiceResult<DecodingKey> {
    let key = read_jwt_public_key_().await?;
    Ok(DecodingKey::from_rsa_pem(key.as_bytes()).map_err_unknown()?)
}

async fn read_jwt_private_key_() -> ServiceResult<String> {
    let path = match std::env::var("JWT_SECRET_KEY_FILE") {
        Ok(path) => path,
        Err(VarError::NotPresent) => panic!("JWT_SECRET_KEY_FILE not set"),
        Err(e) => return Err(ServiceError::unknown(e)),
    };

    let key = tokio::fs::read_to_string(path).await.map_err_unknown()?;

    Ok(key)
}

async fn read_jwt_public_key_() -> ServiceResult<String> {
    let path = match std::env::var("JWT_PUBLIC_KEY_FILE") {
        Ok(path) => path,
        Err(VarError::NotPresent) => panic!("JWT_PUBLIC_KEY_FILE not set"),
        Err(e) => return Err(ServiceError::unknown(e)),
    };

    let key = tokio::fs::read_to_string(path).await.map_err_unknown()?;

    Ok(key)
}
