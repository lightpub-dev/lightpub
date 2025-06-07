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

use actix_web::http::header;
use actix_web::{get, http::StatusCode, web, Responder};
use actix_web::{post, HttpResponse};
use lightpub_service::services::user::{deactivate_user_totp, setup_user_totp, TotpSetup};
use serde::Deserialize;
use url::Url;

use crate::api::auth::middleware_auth_jwt_required;
use crate::template::TotpSetup as TotpSetupTemplate;
use crate::{
    api::auth::AuthedUser,
    template::{
        render_template, Profile, ProfileEdit, ProfileEditBasic, ProfileEditUser, ProfileOg,
        ProfileUser, ProfileUserBasic, Template, UserList,
    },
    AppState,
};
use lightpub_service::{
    services::{
        create_error_simple, create_error_simple_err,
        follow::FollowState,
        user::{get_user_by_id, get_user_id_from_spec, get_user_profile, UserSpecifier},
        ServiceResult,
    },
    utils::sanitize::CleanString,
};

use super::parse_user_spec;

#[get("/register")]
pub async fn client_register_user(st: web::Data<AppState>) -> ServiceResult<impl Responder> {
    if !st.is_registration_open() {
        return create_error_simple(StatusCode::BAD_REQUEST, "registration is closed");
    }

    render_template(st.template(), &Template::Register(()))
}

#[get("/login")]
pub async fn client_login_user(st: web::Data<AppState>) -> ServiceResult<impl Responder> {
    render_template(st.template(), &Template::Login(()))
}

#[get(
    "/changePassword",
    wrap = "from_fn(middleware_redirect_login)",
    wrap = "from_fn(middleware_auth_jwt_optional)"
)]
pub async fn client_change_password(st: web::Data<AppState>) -> ServiceResult<impl Responder> {
    render_template(st.template(), &Template::PasswordChange(()))
}
use crate::client::middleware_auth_jwt_optional;
use crate::client::middleware_redirect_login;
use actix_web::middleware::from_fn;
#[get(
    "/my",
    wrap = "from_fn(middleware_redirect_login)",
    wrap = "from_fn(middleware_auth_jwt_optional)"
)]
pub async fn client_my_profile(
    st: web::Data<AppState>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    // redirect to user profile
    let user_id = auth.user_id_unwrap();

    let user = get_user_by_id(&st.maybe_conn(), &st.rconn(), user_id).await?;
    let user = match user {
        Some(u) => u,
        None => return create_error_simple(StatusCode::NOT_FOUND, "user not found"),
    };

    let url = format!("/client/user/{}", user.specifier());
    Ok(actix_web::HttpResponse::Found()
        .insert_header((actix_web::http::header::LOCATION, url))
        .finish())
}

#[get("/user/{user_spec}", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn client_get_profile(
    st: web::Data<AppState>,
    auth: web::ReqData<AuthedUser>,
    user_spec: web::Path<String>,
) -> ServiceResult<impl Responder> {
    let viewer_id = auth.user_id();
    let data = st.request_data();
    let profile = get_user_profile(
        &st.maybe_conn(),
        &st.rconn(),
        viewer_id,
        &st.my_domain(),
        &parse_user_spec(&user_spec.into_inner(), &st.my_domain())?,
        &data,
    )
    .await?;

    let profile = match profile {
        Some(p) => p,
        None => return create_error_simple(StatusCode::NOT_FOUND, "user not found"),
    };
    // let profile = UserProfileTemplate::from_user_profile(profile, auth.is_authed());

    // let data = serde_json::to_value(profile).unwrap();

    render_template(
        st.template(),
        &Template::Profile(Profile {
            authed: auth.is_authed().into(),
            og: ProfileOg {
                title: profile.basic.nickname.clone(),
                url: st
                    .base_url()
                    .join(format!("/client/user/{}", profile.basic.specifier).as_ref())
                    .unwrap(),
                description: profile.basic.bio.clone(),
                site_name: "Lightpub".to_owned(), // TODO: actual name
                image: profile.basic.avatar.as_ref().map(|up| {
                    st.base_url()
                        .join(format!("/upload/{}", up).as_ref())
                        .unwrap()
                }),
            },
            user: {
                let blocked =
                    profile.is_blocking.unwrap_or(false) && profile.is_blocked.unwrap_or(false);
                let can_follow = profile
                    .is_following
                    .map(|f| profile.followable && !blocked && f == FollowState::No);
                let can_unfollow = profile
                    .is_following
                    .map(|f| profile.followable && !blocked && f != FollowState::No);
                ProfileUser {
                    basic: ProfileUserBasic {
                        id: profile.basic.id,
                        nickname: profile.basic.nickname,
                        specifier: profile.basic.specifier,
                        bio: CleanString::clean(&profile.basic.bio),
                    },
                    follow_count: profile.follow_count,
                    follower_count: profile.follower_count,
                    note_count: profile.note_count,
                    is_following: profile.is_following.map(|f| f == FollowState::Yes),
                    is_followed: profile.is_followed.map(|f| f == FollowState::Yes),
                    is_following_requested: profile.is_following.map(|f| f == FollowState::Pending),
                    is_followed_requested: profile.is_followed.map(|f| f == FollowState::Pending),
                    can_follow,
                    can_unfollow,
                    can_accept_follow: profile.is_followed.map(|f| f == FollowState::Pending),
                    can_refuse_follow: profile.is_followed.map(|f| f != FollowState::No),
                    is_blocked: Some(blocked),

                    view_url: profile
                        .view_url
                        .map(|u| Url::parse(&u).expect("view_url should be valid")),
                    is_me: profile.is_me.into(),
                }
            },
        }),
    )
}

#[get(
    "/user/{user_spec}/edit",
    wrap = "from_fn(middleware_redirect_login)",
    wrap = "from_fn(middleware_auth_jwt_required)"
)]
pub async fn client_edit_profile_get(
    st: web::Data<AppState>,
    user_spec: web::Path<String>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let viewer_id = auth.user_id_unwrap();
    let data = st.request_data();
    let profile = get_user_profile(
        &st.maybe_conn(),
        &st.rconn(),
        Some(viewer_id),
        &st.my_domain(),
        &parse_user_spec(&user_spec.into_inner(), &st.my_domain())?,
        &data,
    )
    .await?;

    let profile = match profile {
        Some(p) => p,
        None => return create_error_simple(StatusCode::NOT_FOUND, "user not found"),
    };

    if profile.basic.id != viewer_id {
        return create_error_simple(StatusCode::FORBIDDEN, "not your account");
    }

    let edit_data = Template::ProfileEdit(ProfileEdit {
        user: ProfileEditUser {
            basic: ProfileEditBasic {
                id: profile.basic.id,
                bio: profile.basic.bio,
                nickname: profile.basic.nickname,
            },
            auto_follow_accept: profile.auto_follow_accept,
            hide_follows: profile.hide_follows,
        },
    });

    render_template(st.template(), &edit_data)
}

#[get("/user/{user_spec}/following")]
pub async fn client_user_followings_list(
    st: web::Data<AppState>,
    user_spec: web::Path<String>,
) -> ServiceResult<impl Responder> {
    let user_spec =
        UserSpecifier::from_str(&user_spec.into_inner(), &st.my_domain()).ok_or_else(|| {
            create_error_simple_err(StatusCode::BAD_REQUEST, "invalid user specifier")
        })?;
    let user_id = match get_user_id_from_spec(&st.maybe_conn(), &user_spec, &st.my_domain()).await?
    {
        None => return create_error_simple(StatusCode::NOT_FOUND, "user not found"),
        Some(u) => u.to_owned(),
    };

    render_template(
        st.template(),
        &Template::UserList(UserList {
            title: "フォロー一覧".to_owned(),
            url: format!("/user/{}/following", user_id),
        }),
    )
}

#[get("/user/{user_spec}/followers")]
pub async fn client_user_followers_list(
    st: web::Data<AppState>,
    user_spec: web::Path<String>,
) -> ServiceResult<impl Responder> {
    let user_spec =
        UserSpecifier::from_str(&user_spec.into_inner(), &st.my_domain()).ok_or_else(|| {
            create_error_simple_err(StatusCode::BAD_REQUEST, "invalid user specifier")
        })?;
    let user_id = match get_user_id_from_spec(&st.maybe_conn(), &user_spec, &st.my_domain()).await?
    {
        None => return create_error_simple(StatusCode::NOT_FOUND, "user not found"),
        Some(u) => u.to_owned(),
    };

    render_template(
        st.template(),
        &Template::UserList(UserList {
            title: "フォロワー一覧".to_owned(),
            url: format!("/user/{}/followers", user_id),
        }),
    )
}

#[derive(Debug, Clone, Deserialize)]
pub struct TotpSetupConfig {
    #[serde(default)]
    pub force: bool,
}

#[post(
    "/config/totp/setup",
    wrap = "from_fn(middleware_redirect_login)",
    wrap = "from_fn(middleware_auth_jwt_required)"
)]
pub async fn client_user_totp_setup(
    st: web::Data<AppState>,
    config: web::Form<TotpSetupConfig>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let viewer_id = auth.user_id_unwrap();

    let totp = setup_user_totp(
        &st.conn(),
        &st.rconn(),
        viewer_id,
        config.force,
        &st.my_domain(),
    )
    .await?;

    match totp {
        TotpSetup::Success {
            qr_code_png_base64,
            url: _,
        } => render_template(
            st.template(),
            &Template::TotpSetup(TotpSetupTemplate {
                qr_base64: Some(qr_code_png_base64),
                success: true,
            }),
        ),
        TotpSetup::AlreadySetup => render_template(
            st.template(),
            &Template::TotpSetup(TotpSetupTemplate {
                qr_base64: None,
                success: false,
            }),
        ),
    }
}

#[post(
    "/config/totp/deactivate",
    wrap = "from_fn(middleware_redirect_login)",
    wrap = "from_fn(middleware_auth_jwt_required)"
)]
pub async fn client_user_totp_deactivate(
    st: web::Data<AppState>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let viewer_id = auth.user_id_unwrap();

    deactivate_user_totp(&st.conn(), &st.rconn(), viewer_id).await?;

    Ok(HttpResponse::Found()
        .insert_header((header::LOCATION, "/client/my"))
        .finish())
}
