use activitypub_federation::config::Data;
use itertools::Itertools;
use sea_orm::{Condition, EntityTrait};
use sea_orm::{QueryFilter, QuerySelect};
use url::Url;

use crate::MyFederationData;
use crate::services::MapToUnknown;
use crate::services::user::{UserSpecifier, get_user_by_id, get_user_by_spec_with_remote};
use crate::{
    ServiceResult,
    services::{
        db::MaybeTxConn,
        id::{Identifier, UserID},
        kv::KVObject,
        user::SimpleUserModel,
    },
};
use sea_orm::ColumnTrait;

fn parse_username_and_domain(text: &str) -> Option<(&str, Option<&str>)> {
    let (username, domain) = {
        let parts: Vec<_> = text.split('@').collect();
        if parts.len() == 0 {
            unreachable!();
        } else if parts.len() == 1 {
            (parts[0], None)
        } else if parts.len() == 2 {
            if parts[0] == "" {
                (parts[1], None)
            } else {
                (parts[0], Some(parts[1]))
            }
        } else if parts.len() == 3 {
            (parts[1], Some(parts[2]))
        } else {
            return None;
        }
    };
    Some((username, domain))
}

#[test]
fn test_parse_username_and_domain() {
    let p = parse_username_and_domain;
    assert_eq!(p("username"), Some(("username", None)));
    assert_eq!(p("@username"), Some(("username", None)));
    assert_eq!(p("username@"), Some(("username", Some(""))));
    assert_eq!(p("username@domain"), Some(("username", Some("domain"))));
    assert_eq!(p("@username@domain"), Some(("username", Some("domain"))));
}

pub async fn search_user_by_text(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    text: &str,
    fuzzy: bool,
    allow_remote: bool,
    allow_http: bool,
    my_domain: &str,
    data: &Data<MyFederationData>,
) -> ServiceResult<Vec<SimpleUserModel>> {
    if text == "" {
        return Ok(Vec::new());
    }

    if text.contains("%") {
        return Ok(Vec::new());
    }

    let maybe_user_id = UserID::from_string(text);

    let maybe_url = {
        let parsed = Url::parse(text).ok();
        match parsed {
            Some(parsed)
                if parsed.scheme() == "https" || (parsed.scheme() == "http" && allow_http) =>
            {
                Some(parsed)
            }
            _ => None,
        }
    };

    let username_opt = match maybe_url {
        Some(_) => None, // URL としてパースできたならユーザー名ではない
        None => parse_username_and_domain(text),
    };
    let username = {
        let username = username_opt.map(|u| u.0);
        match username {
            Some(username) if username.len() > 0 => Some(username),
            _ => None,
        }
    };
    let domain = {
        let mut dom = username_opt.map(|u| u.1).flatten();
        if dom.is_some_and(|d| d == my_domain) {
            dom = Some("");
        }
        dom
    };

    let cond = if fuzzy {
        let username = username.map(|u| format!("{}%", u));
        Condition::any()
            .add_option(maybe_user_id.map(|d| entity::user::Column::Id.eq(d.as_db())))
            .add_option((username.is_some() || domain.is_some()).then(|| {
                Condition::all()
                    .add_option(
                        username.map(|username| entity::user::Column::Username.like(username)),
                    )
                    .add_option(domain.map(|domain| entity::user::Column::Domain.eq(domain)))
            }))
            .add_option(
                maybe_url
                    .as_ref()
                    .map(|url| entity::user::Column::Url.eq(url.as_str())),
            )
    } else {
        Condition::any()
            .add_option(maybe_user_id.map(|d| entity::user::Column::Id.eq(d.as_db())))
            .add_option((username.is_some() || domain.is_some()).then(|| {
                Condition::all()
                    .add_option(
                        username.map(|username| entity::user::Column::Username.eq(username)),
                    )
                    .add_option(domain.map(|domain| entity::user::Column::Domain.eq(domain)))
            }))
            .add_option(
                maybe_url
                    .as_ref()
                    .map(|url| entity::user::Column::Url.eq(url.as_str())),
            )
    };

    let user_ids: Vec<_> = entity::user::Entity::find()
        .filter(cond)
        .limit(30) // TODO: pagination?
        .all(conn)
        .await
        .map_err_unknown()?
        .into_iter()
        .map(|u| UserID::from_db_trusted(u.id))
        .collect();

    let mut users = Vec::new();
    for user_id in user_ids {
        let user = get_user_by_id(conn, rconn, user_id)
            .await?
            .expect("user not found");
        users.push(user);
    }

    if allow_remote {
        match (username, domain, maybe_url) {
            (Some(username), Some(domain), _) => {
                let remote_user = get_user_by_spec_with_remote(
                    conn,
                    rconn,
                    &UserSpecifier::Username(username.to_string(), Some(domain.to_string())),
                    my_domain,
                    data,
                )
                .await?;
                if let Some(ru) = remote_user {
                    users.push(ru);
                }
            }
            (_, _, Some(url)) => {
                let remote_user = get_user_by_spec_with_remote(
                    conn,
                    rconn,
                    &UserSpecifier::url(url),
                    my_domain,
                    data,
                )
                .await?;
                if let Some(ru) = remote_user {
                    users.push(ru);
                }
            }
            _ => {}
        }
    }

    // 重複を省く
    let users = users.into_iter().unique_by(|u| u.id).collect();

    Ok(users)
}
