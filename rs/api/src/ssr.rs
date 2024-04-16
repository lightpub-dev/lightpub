use actix_web::{cookie::Cookie, get, http::header, post, web, HttpResponse, Responder};
use askama_actix::Template;
use lightpub_backend::{
    db::new_user_service, ServiceError, UserCreateError, UserCreateRequest, UserLoginError,
    UserLoginRequest,
};
use serde::Deserialize;

use crate::{state::AppState, validate_password, validate_username};

pub const SSR_BASE_URL: &str = "/nojs";
fn make_ssr_url(path: &str) -> String {
    format!("{}/{}", SSR_BASE_URL, path)
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    error: Option<String>,
}

#[get("/login")]
async fn login_get() -> impl Responder {
    LoginTemplate { error: None }
}

#[derive(Debug, Deserialize)]
struct LoginPost {
    username: String,
    password: String,
}

#[post("/login")]
async fn login_post(body: web::Form<LoginPost>, data: web::Data<AppState>) -> impl Responder {
    let mut us = new_user_service(data.pool().clone());
    let req = us
        .login_user(&UserLoginRequest {
            username: body.username.clone(),
            password: body.password.clone(),
        })
        .await;
    match req {
        Ok(req) => HttpResponse::SeeOther()
            .cookie(
                Cookie::build("LoginToken", req.user_token().to_string())
                    .secure(true)
                    .http_only(true)
                    .same_site(actix_web::cookie::SameSite::Strict)
                    .finish(),
            )
            .insert_header((header::LOCATION, make_ssr_url("/timeline")))
            .finish(),
        Err(e) => match e {
            ServiceError::SpecificError(e) => match e {
                UserLoginError::AuthFailed => {
                    let t = LoginTemplate {
                        error: Some("Invalid username or password".to_string()),
                    };
                    HttpResponse::Unauthorized()
                        .content_type(LoginTemplate::MIME_TYPE)
                        .body(t.render().unwrap())
                }
            },
            _ => {
                let t = LoginTemplate {
                    error: Some("Internal server error".to_string()),
                };
                HttpResponse::InternalServerError()
                    .content_type(LoginTemplate::MIME_TYPE)
                    .body(t.render().unwrap())
            }
        },
    }
}

#[get("/register")]
async fn register_get() -> impl Responder {
    RegisterTemplate { error: None }
}

#[derive(Debug, Deserialize)]
struct RegisterPost {
    username: String,
    nickname: String,
    password: String,
}

#[post("/register")]
async fn register_post(body: web::Form<RegisterPost>, data: web::Data<AppState>) -> impl Responder {
    if !data.config().instance.open_registration {
        let t = RegisterTemplate {
            error: Some("Registration is closed".to_string()),
        };
        return HttpResponse::Forbidden()
            .content_type(RegisterTemplate::MIME_TYPE)
            .body(t.render().unwrap());
    }

    if !validate_username(&body.username) {
        let t = RegisterTemplate {
            error: Some("Invalid username".to_string()),
        };
        return HttpResponse::BadRequest()
            .content_type(RegisterTemplate::MIME_TYPE)
            .body(t.render().unwrap());
    }
    if !validate_password(&body.password) {
        let t = RegisterTemplate {
            error: Some("Invalid password".to_string()),
        };
        return HttpResponse::BadRequest()
            .content_type(RegisterTemplate::MIME_TYPE)
            .body(t.render().unwrap());
    }

    let mut us = new_user_service(data.pool().clone());
    let req = us
        .create_user(&UserCreateRequest {
            username: body.username.clone(),
            nickname: body.nickname.clone(),
            password: body.password.clone(),
        })
        .await;
    match req {
        Ok(_req) => HttpResponse::SeeOther()
            .insert_header((header::LOCATION, make_ssr_url("/login")))
            .finish(),
        Err(e) => match e {
            ServiceError::SpecificError(e) => match e {
                UserCreateError::UsernameConflict => {
                    let t = RegisterTemplate {
                        error: Some("Username already exists".to_string()),
                    };
                    HttpResponse::Conflict()
                        .content_type(RegisterTemplate::MIME_TYPE)
                        .body(t.render().unwrap())
                }
            },
            _ => {
                let t = RegisterTemplate {
                    error: Some("Internal server error".to_string()),
                };
                HttpResponse::InternalServerError()
                    .content_type(RegisterTemplate::MIME_TYPE)
                    .body(t.render().unwrap())
            }
        },
    }
}
