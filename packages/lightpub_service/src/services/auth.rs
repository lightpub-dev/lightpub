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

use crate::try_opt_res;

use super::{
    MapToUnknown, ServiceError, ServiceResult, create_error_simple,
    db::Conn,
    id::{Identifier, UserID},
};
use activitypub_federation::http_signatures::generate_actor_keypair;
use actix_web::http::StatusCode;
use chrono::Utc;
use expected_error_derive::ExpectedError;
use regex::Regex;
use sea_orm::{ActiveModelTrait, SqlErr};
use sea_orm::{Condition, IntoActiveModel};
use sea_orm::{Set, TransactionTrait, prelude::*};
use thiserror::Error;

#[derive(Debug)]
pub struct RegisterUserResult {
    pub user_id: UserID,
}

#[derive(Error, Debug, ExpectedError)]
pub enum UserRegistrationError {
    #[error("Username already exists")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    UsernameAlreadyExists,
}

pub fn validate_username(username: &str) -> ServiceResult<()> {
    let re = Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
    if !re.is_match(username) {
        return create_error_simple(StatusCode::BAD_REQUEST, "invalid username");
    }
    Ok(())
}

pub fn validate_nickname(nickname: &str) -> ServiceResult<()> {
    if nickname.len() < 1 || nickname.len() > 50 {
        return create_error_simple(StatusCode::BAD_REQUEST, "invalid nickname");
    }
    Ok(())
}

fn validate_password(password: &str) -> ServiceResult<()> {
    if password.len() < 8 || password.len() >= 64 {
        return create_error_simple(StatusCode::BAD_REQUEST, "invalid password");
    }
    Ok(())
}

pub async fn register_user(
    conn: &Conn,
    username: &str,
    nickname: &str,
    password: &str,
) -> ServiceResult<RegisterUserResult> {
    validate_username(username)?;
    validate_nickname(nickname)?;
    validate_password(password)?;

    let password = password.to_string();
    let hashed_password_res = tokio::task::spawn_blocking(move || {
        bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err_unknown()
    })
    .await
    .map_err_unknown()?;
    let hashed_password = hashed_password_res?;

    let key_pair_res = tokio::task::spawn_blocking(move || generate_actor_keypair())
        .await
        .map_err_unknown()?;
    let key_pair = key_pair_res.map_err_unknown()?;

    let user_id = UserID::new_random();

    let user = entity::user::ActiveModel {
        id: Set(user_id.as_db()),
        username: Set(username.to_string()),
        domain: Set("".into()),
        nickname: Set(nickname.to_string()),
        password: Set(hashed_password.into()),
        bio: Set("".into()),
        private_key: Set(Some(key_pair.private_key)),
        public_key: Set(Some(key_pair.public_key)),
        created_at: Set(Some(Utc::now().naive_utc())),
        ..Default::default()
    };
    match user.insert(conn.db()).await {
        Ok(_) => {}
        Err(e) => match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                return Err(ServiceError::known(
                    UserRegistrationError::UsernameAlreadyExists,
                ));
            }
            _ => {
                return Err(ServiceError::unknown(e));
            }
        },
    }

    Ok(RegisterUserResult { user_id })
}

pub async fn change_password(
    conn: &Conn,
    user_id: UserID,
    new_password: &str,
    expire_all: bool,
) -> ServiceResult<()> {
    validate_password(new_password)?;

    let hashed_password = bcrypt::hash(new_password, bcrypt::DEFAULT_COST).map_err_unknown()?;

    let user = entity::user::Entity::find_by_id(user_id.as_db())
        .one(conn.db())
        .await
        .map_err_unknown()?;

    let mut user = match user {
        None => return Err(ServiceError::ise("user not found")),
        Some(u) => u,
    }
    .into_active_model();
    user.password = Set(hashed_password.into());
    user.update(conn.db()).await.map_err_unknown()?;

    if expire_all {
        logout_all(conn, user_id).await?;
    }

    Ok(())
}

#[derive(Debug)]
pub struct LoginResult {
    pub user_id: UserID,
}

pub async fn check_password_user(
    conn: &Conn,
    username: &str,
    password: &str,
) -> ServiceResult<Option<LoginResult>> {
    let user = entity::user::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user::Column::Username.eq(username))
                .add(entity::user::Column::Domain.eq("")),
        )
        .one(conn.db())
        .await
        .map_err(ServiceError::unknown)?;

    let user = match user {
        Some(u) => u,
        None => return Ok(None),
    };

    let hashed_password = match user.password {
        Some(p) => p,
        None => return Ok(None), // probably remote user
    };

    if bcrypt::verify(password, &hashed_password).map_err(ServiceError::unknown)? {
        Ok(Some(LoginResult {
            user_id: UserID::from_db_trusted(user.id),
        }))
    } else {
        Ok(None)
    }
}

pub async fn logout_all(conn: &Conn, user_id: UserID) -> ServiceResult<()> {
    let txn = conn.db().begin().await.map_err_unknown()?;

    let user = entity::user::Entity::find_by_id(user_id.as_db())
        .one(&txn)
        .await
        .map_err_unknown()?;

    if let Some(user) = user {
        let mut user = user.into_active_model();
        user.auth_expired_at = Set(Some(Utc::now().naive_utc()));
        user.update(&txn).await.map_err_unknown()?;
    }

    txn.commit().await.map_err_unknown()?;

    Ok(())
}

/// returns true if the login is still valid
pub async fn check_user_login_expiration(
    conn: &Conn,
    user_id: &UserID,
    logged_in_at: ChronoDateTimeUtc,
) -> ServiceResult<Option<bool>> {
    let user = try_opt_res!(
        entity::user::Entity::find_by_id(user_id.as_db())
            .one(conn.db())
            .await
            .map_err_unknown()?
    );

    if let Some(expired_at) = user.auth_expired_at {
        if expired_at < logged_in_at.naive_utc() {
            Ok(Some(true))
        } else {
            Ok(Some(false))
        }
    } else {
        Ok(Some(true))
    }
}
