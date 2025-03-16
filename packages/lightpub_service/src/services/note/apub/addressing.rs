use std::collections::HashSet;

use url::Url;

use crate::{
    ServiceResult,
    services::{
        MapToUnknown,
        apub::PUBLIC_URL,
        db::MaybeTxConn,
        id::{Identifier, NoteID, UserID},
        note::VisibilityModel,
        user::get_apubuser_by_id,
    },
};
use sea_orm::{Condition, prelude::*};

#[derive(Debug, Clone)]
pub struct CalculateToAndCcResult {
    pub to: Vec<Url>,
    pub cc: Vec<Url>,
    pub inboxes: Vec<Url>,
}

pub async fn calculate_to_and_cc_of_renote(
    tx: &MaybeTxConn,
    renote_author_id: UserID,
    renote_target_author_id: UserID,
    visibility: VisibilityModel,
    base_url: &Url,
) -> ServiceResult<CalculateToAndCcResult> {
    let (renote_followers_url, renote_followers_inboxes) =
        calculate_follower_recipients(tx, renote_author_id, base_url).await?;

    let (renote_target_url, renote_target_inbox) =
        calculate_user_addresses(tx, renote_target_author_id, base_url).await?;

    let mut to = vec![];
    let mut cc = vec![];
    if let Some(follower_url) = renote_followers_url {
        to.push(follower_url);
    }
    if let Some(renote_target_url) = renote_target_url {
        to.push(renote_target_url);
    }
    match visibility {
        VisibilityModel::Public => {
            to.push((*&PUBLIC_URL).clone());
        }
        VisibilityModel::Unlisted => {
            cc.push((*&PUBLIC_URL).clone());
        }
        _ => unreachable!("visibility should be public or unlisted"),
    }

    let inboxes = deduplicate_urls(vec![
        renote_followers_inboxes,
        renote_target_inbox
            .map(|u| vec![u])
            .unwrap_or_else(|| Vec::new()),
    ]);

    Ok(CalculateToAndCcResult { to, cc, inboxes })
}

async fn calculate_mention_recipients(
    tx: &MaybeTxConn,
    note_id: NoteID,
    base_url: &Url,
) -> ServiceResult<(Vec<Url>, Vec<Url>)> {
    let mentions = entity::note_mention::Entity::find()
        .filter(entity::note_mention::Column::NoteId.eq(note_id.as_db()))
        .all(tx)
        .await
        .map_err_unknown()?;

    let mut mention_urls = vec![];
    let mut mention_inboxes = vec![];
    for mention in mentions {
        let user = get_apubuser_by_id(
            tx,
            UserID::from_db_trusted(mention.target_user_id),
            base_url,
        )
        .await
        .map_err_unknown()?
        .expect("mentioned user should exist");
        mention_urls.push(user.apub.url.clone());
        mention_inboxes.push(user.shared_inbox_or_inbox().clone());
    }

    Ok((mention_urls, mention_inboxes))
}

async fn calculate_follower_recipients(
    tx: &MaybeTxConn,
    author_id: UserID,
    base_url: &Url,
) -> ServiceResult<(Option<Url>, Vec<Url>)> {
    let author_model = get_apubuser_by_id(tx, author_id, base_url)
        .await?
        .expect("author should exist");
    let author_followers_url = author_model.apub.followers.clone();

    let followers = entity::user_follow::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user_follow::Column::FollowedId.eq(author_model.basic.id.as_db()))
                .add(entity::user_follow::Column::Pending.eq(false)),
        )
        .all(tx)
        .await
        .map_err_unknown()?;

    let mut followers_inboxes = vec![];
    for follower in followers {
        let follower =
            get_apubuser_by_id(tx, UserID::from_db_trusted(follower.follower_id), base_url)
                .await
                .map_err_unknown()?
                .expect("follower should exist");
        followers_inboxes.push(follower.shared_inbox_or_inbox().clone());
    }

    Ok((author_followers_url, followers_inboxes))
}

async fn calculate_user_addresses(
    tx: &MaybeTxConn,
    user_id: UserID,
    base_url: &Url,
) -> ServiceResult<(Option<Url>, Option<Url>)> {
    let user_model = get_apubuser_by_id(tx, user_id, base_url)
        .await?
        .expect("user should exist");

    let inbox = if user_model.is_remote() {
        Some(user_model.shared_inbox_or_inbox().clone())
    } else {
        None
    };

    // Return the user's URL that can be used in to/cc addressing
    let address = Some(user_model.apub.url.clone());

    Ok((address, inbox))
}

pub async fn calculate_to_and_cc(
    tx: &MaybeTxConn,
    note_id: NoteID,
    author_id: UserID,
    visibility: VisibilityModel,
    include_author: bool,
    base_url: &Url,
) -> ServiceResult<CalculateToAndCcResult> {
    // Calculate mentions
    let (mention_urls, mention_inboxes) =
        calculate_mention_recipients(tx, note_id, base_url).await?;

    // Calculate followers
    let (author_followers_url, followers_inboxes) =
        calculate_follower_recipients(tx, author_id, base_url).await?;

    // Calculate author addresses if needed
    let (_, author_inbox) = if include_author {
        let result = calculate_user_addresses(tx, author_id, base_url).await?;
        result
    } else {
        (None, None)
    };

    let mut to = vec![];
    let mut cc = vec![];

    // Add mentions to "to"
    to.extend(mention_urls);

    // Add visibility-based recipients
    match visibility {
        VisibilityModel::Public => {
            to.push(PUBLIC_URL.clone());
            if let Some(f) = author_followers_url {
                cc.push(f);
            }
        }
        VisibilityModel::Unlisted => {
            cc.push(PUBLIC_URL.clone());
            if let Some(f) = author_followers_url.clone() {
                cc.push(f);
            }
        }
        VisibilityModel::Follower => {
            if let Some(f) = author_followers_url {
                to.push(f);
            }
        }
        VisibilityModel::Private => {}
    }

    // Deduplicate and collect inboxes
    let mut inbox_sources = vec![mention_inboxes, followers_inboxes];
    if let Some(inbox) = author_inbox {
        inbox_sources.push(vec![inbox]);
    }
    let inboxes = deduplicate_urls(inbox_sources);

    Ok(CalculateToAndCcResult { to, cc, inboxes })
}

fn deduplicate_urls(url_vectors: Vec<Vec<Url>>) -> Vec<Url> {
    let mut result = vec![];
    let mut url_hash = HashSet::new();

    for vector in url_vectors {
        for url in vector {
            if url_hash.insert(url.clone()) {
                result.push(url);
            }
        }
    }

    result
}
