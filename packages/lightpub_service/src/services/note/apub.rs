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

use std::str::FromStr;

use super::BasicNoteModel;
use super::ContentType;
use super::Mention;
use super::NoteSpecifier;
use super::NoteUpload;
use super::PostCreateOptionsBuilder;
use super::VisibilityModel;
use super::create::ExistingNote;
use super::create::upsert_note;
use super::create::validate_note_content;
use super::delete;
use super::get::get_apubnote_by_id;
use super::get::get_apubnote_by_spec;
use super::upload::NoteUploadApubModel;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::kinds::object::NoteType;
use activitypub_federation::traits::Object;
use actix_web::http::StatusCode;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_getters::Getters;
use expected_error_derive::ExpectedError;
use mime::Mime;
use nestify::nest;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use tracing::warn;
use url::Url;

use crate::MyFederationData;
use crate::services::ServiceError;
use crate::services::UpsertOperation;
use crate::services::apub::AnnounceActivity;
use crate::services::apub::CreatableObject;
use crate::services::apub::CreateActivity;
use crate::services::apub::contains_public_url;
use crate::services::db::Conn;
use crate::services::db::MaybeTxConn;
use crate::services::id::UploadID;
use crate::services::kv::KVObject;
use crate::services::queue::QConn;
use crate::services::user::get_apubuser_by_id;

use super::super::FederationServiceError;
use super::super::ServiceResult;
use super::super::create_error_simple;
use super::super::user::UserWithApubModel;

mod addressing;

pub use addressing::{CalculateToAndCcResult, calculate_to_and_cc, calculate_to_and_cc_of_renote};

nest! {
    #[derive(Debug, Clone)]*
    pub struct NoteWithApubModel {
        pub(crate) basic: BasicNoteModel,
        pub(crate) apub: pub struct NoteApubData {
            pub(crate) url: Url,
            pub(crate) view_url: Option<Url>,
            pub(crate) author_url: Url,
            pub(crate) in_reply_to_url: Option<Url>,
            pub(crate) fetched_at: Option<DateTime<Utc>>,
            pub(crate) to: Vec<Url>,
            pub(crate) cc: Vec<Url>,
            pub(crate) mentions: Vec<pub struct NoteApubMentionData {
                pub(crate) url: Url,
                pub(crate) name: String,
            }>,
            pub(crate) hashtags: Vec<pub struct NoteApubHashtagData {
                pub(crate) name: String,
                pub(crate) url: Url,
            }>,
            pub(crate) attachments: Vec<NoteUploadApubModel>,
        }
    }
}

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum ApubNoteError {
    #[error("Bad domain")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    BadDomain,
}

#[async_trait]
impl Object for NoteWithApubModel {
    type DataType = MyFederationData;
    type Kind = ApubNoteModel;
    type Error = FederationServiceError;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        self.apub.fetched_at.clone()
    }

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let note = get_apubnote_by_spec(
            &data.maybe_conn(),
            &NoteSpecifier::url(object_id),
            &data.my_domain(),
            data.base_url(),
        )
        .await
        .map_err(|e| FederationServiceError::ServiceError(e))?;

        Ok(note)
    }

    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let note_id = self.basic.id;
        delete::delete_note_by_id_(data.conn(), data.qconn(), note_id, None, data.base_url())
            .await?;
        Ok(())
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let mut tags = vec![];
        for tag in self.apub.hashtags {
            tags.push(ApubNoteTagModel {
                kind: ApubTagType::Hashtag,
                href: tag.url,
                name: Some(tag.name),
            })
        }
        for mention in self.apub.mentions {
            tags.push(ApubNoteTagModel {
                kind: ApubTagType::Mention,
                href: mention.url,
                name: Some(mention.name),
            })
        }

        let attachment = self
            .apub
            .attachments
            .into_iter()
            .map(|a| ApubNoteAttachment {
                kind: a.kind,
                media_type: a.mime_type,
                url: a.url,
            })
            .collect();

        let note_content = &self.basic.content.expect("content must be set");
        let source = ApubNoteSourceModel {
            content: note_content.as_raw_text().to_string(),
            media_type: note_content.mime_type().to_string(),
        };

        let qconn = data.qconn();
        Ok(ApubNoteModel {
            id: ObjectId::parse(&self.apub.url.to_string()).unwrap(),
            attributed_to: ObjectId::parse(&self.apub.author_url.to_string()).unwrap(),
            content: note_content.render_to_html(qconn).await?.into_inner(),
            kind: NoteType::Note,
            published: self.basic.created_at,
            updated: self.basic.updated_at,
            sensitive: Some(self.basic.sensitive),
            in_reply_to: self
                .apub
                .in_reply_to_url
                .map(|u| ObjectId::parse(&u.to_string()).unwrap()),
            source: Some(source),
            tags: Some(tags),
            url: self.apub.view_url,
            to: self.apub.to,
            cc: self.apub.cc,
            attachment: Some(attachment),
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        let object_id = json.id.inner();
        let object_id_domain = object_id
            .domain()
            .ok_or(ServiceError::known(ApubNoteError::BadDomain))?;
        if object_id_domain != expected_domain.domain().unwrap() {
            return Err(ServiceError::known(ApubNoteError::BadDomain).into());
        }

        json.validate()?;

        Ok(())
    }

    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        upsert_apub_note(
            data.conn(),
            &data.rconn(),
            data.qconn(),
            data,
            &json,
            &data.my_domain(),
            data.base_url(),
            data,
        )
        .await
        .map_err(|e| e.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApubTagType {
    #[serde(rename = "Hashtag")]
    Hashtag,
    #[serde(rename = "Mention")]
    Mention,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum ApubAttachmentType {
//     #[serde(rename = "Document")]

// }

nest! {
    #[derive(Debug, Clone, Serialize, Deserialize, Getters)]*
    #[serde(rename_all = "camelCase")]*
    pub struct ApubNoteModel {
        pub(crate) id: ObjectId<NoteWithApubModel>,
        #[serde(rename = "type")]
        pub(crate) kind: NoteType,
        pub(crate) attributed_to: ObjectId<UserWithApubModel>,
        pub(crate) content: String,
        pub(crate) published: DateTime<Utc>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) updated: Option<DateTime<Utc>>,
        #[serde(default)]
        pub(crate) to: Vec<Url>,
        #[serde(default)]
        pub(crate) cc: Vec<Url>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) url: Option<Url>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) source: Option<pub struct ApubNoteSourceModel {
            pub(crate) content: String,
            pub(crate) media_type: String
        }>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) in_reply_to: Option<ObjectId<UserWithApubModel>>,
        pub(crate) sensitive: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "tag")]
        pub(crate) tags: Option<Vec<pub struct ApubNoteTagModel {
            #[serde(rename = "type")]
            pub(crate) kind: ApubTagType,
            pub(crate) name: Option<String>,
            pub(crate) href: Url,
        }>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) attachment: Option<Vec<pub struct ApubNoteAttachment {
            #[serde(rename = "type")]
            pub(crate) kind: String,
            pub(crate) url: Url,
            pub(crate) media_type: String,
        }>>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ApubNoteTag {
    #[serde(rename = "Hashtag")]
    Hashtag { name: Option<String>, href: Url },
    #[serde(rename = "Mention")]
    Mention { name: Option<String>, href: Url },
    #[serde(rename = "Emoji")]
    Emoji {
        // TODO: ignored for now
    },
}

impl ApubNoteModel {
    pub fn validate(&self) -> ServiceResult<()> {
        validate_note_content(&self.content)?;

        if let Some(attachments) = self.attachment.as_ref() {
            let domain = self.id.inner().domain().expect("domain must be set");
            for at in attachments {
                at.validate(domain)?;
            }
        }

        Ok(())
    }
}

impl ApubNoteAttachment {
    pub fn validate(&self, domain: &str) -> ServiceResult<()> {
        if self.url.domain() != Some(domain) {
            return create_error_simple(
                StatusCode::BAD_REQUEST,
                "attachment is not in the same domain",
            );
        }

        Ok(())
    }
}

async fn upsert_apub_note(
    conn: &Conn,
    rconn: &KVObject,
    qconn: &QConn,
    data: &Data<MyFederationData>,
    json: &ApubNoteModel,
    my_domain: &str,
    base_url: &Url,
    fed_data: &Data<MyFederationData>,
) -> ServiceResult<NoteWithApubModel> {
    let author = json.attributed_to.dereference(data).await?;
    let author_id = author.basic.id;
    let content = &json.content;

    let to = json.to.clone();
    let cc = json.cc.clone();
    let viz = infer_apub_visibility(&to, &cc);

    let mut hashtags = vec![];
    let mut mentions = vec![];
    if let Some(tags) = &json.tags {
        for tag in tags {
            match tag.kind {
                ApubTagType::Mention => {
                    let user_url = &tag.href;
                    mentions.push(Mention::ByURL(user_url.clone()));
                }
                ApubTagType::Hashtag => {
                    let hashtag = &tag.name;
                    if let Some(hashtag) = hashtag.as_ref() {
                        hashtags.push(hashtag.clone());
                    } else {
                        warn!("empty hashtag");
                    }
                }
            }
        }
    }

    let mut uploads = vec![];
    if let Some(attachments) = &json.attachment {
        for at in attachments {
            let mime_type = match Mime::from_str(&at.media_type) {
                Ok(m) => match m.type_() {
                    mime::IMAGE => m,
                    ty => {
                        warn!("unsupported media type (skip): {}", ty);
                        continue;
                    }
                },
                Err(_) => {
                    warn!("invalid mime type: {}", at.media_type);
                    continue;
                }
            };
            let upload_id = UploadID::new_random();
            uploads.push(NoteUpload::URL(
                upload_id,
                at.url.clone(),
                mime_type.to_string(),
            ))
        }
    }

    let note_id = upsert_note(
        conn,
        rconn,
        qconn,
        Some(ExistingNote::ByURL(json.id.inner().clone())),
        author_id,
        content,
        ContentType::Html,
        Some(viz),
        &PostCreateOptionsBuilder::default()
            .sensitive(UpsertOperation::Set(json.sensitive.unwrap_or(false)))
            .mentions_override(Some(mentions))
            .hashtags_override(Some(hashtags))
            .created_at(Some(json.published))
            .uploads(UpsertOperation::Set(uploads))
            .build()
            .unwrap(),
        my_domain,
        base_url,
        fed_data,
    )
    .await?;

    let conn = conn.clone().into();
    let note =
        get_apubnote_by_spec(&conn, &NoteSpecifier::ID(note_id), my_domain, base_url).await?;
    Ok(note.expect("upserted note not found"))
}

fn infer_apub_visibility(to: &[Url], cc: &[Url]) -> VisibilityModel {
    if contains_public_url(to) {
        return VisibilityModel::Public;
    } else if contains_public_url(cc) {
        return VisibilityModel::Unlisted;
    }

    for t in to {
        if t.path().ends_with("/followers") {
            return VisibilityModel::Follower;
        }
    }

    VisibilityModel::Private
}

#[derive(Debug, Clone, Serialize)]
pub enum OutboxActivity {
    Create(CreateActivity),
    Announce(AnnounceActivity),
}

impl OutboxActivity {
    pub fn published(&self) -> DateTime<Utc> {
        match self {
            OutboxActivity::Create(c) => match &c.object {
                CreatableObject::Note(n) => n.published,
            },
            OutboxActivity::Announce(a) => a.published,
        }
    }

    pub async fn from_note_apub(
        note: NoteWithApubModel,
        conn: &MaybeTxConn,
        data: &Data<MyFederationData>,
    ) -> ServiceResult<Self> {
        let base_url = data.base_url();

        // check if the note is renote
        match note.basic.content.as_ref() {
            Some(_) => Ok(Self::Create(CreateActivity::from_note(
                note.into_json(data).await?,
            ))),
            None => {
                let object_id = note.basic.renote_of_id.expect("should be renote");
                let renoted = get_apubnote_by_id(conn, object_id, base_url, true)
                    .await?
                    .expect("should exist"); // TODO: better None handling
                let renoted_url = renoted.apub.url;
                let renoted_user = get_apubuser_by_id(conn, renoted.basic.author.id, base_url)
                    .await?
                    .expect("author should exist");
                let toandcc = calculate_to_and_cc_of_renote(
                    conn,
                    note.basic.author.id,
                    renoted.basic.author.id,
                    note.basic.visibility,
                    base_url,
                )
                .await?;
                Ok(Self::Announce(AnnounceActivity::from_note(
                    ObjectId::from(renoted_url),
                    note.apub.url,
                    ObjectId::from(renoted_user.apub.url),
                    toandcc.to.into_iter().map(ObjectId::from).collect(),
                    toandcc.cc.into_iter().map(ObjectId::from).collect(),
                    note.basic.created_at,
                )))
            }
        }
    }
}
