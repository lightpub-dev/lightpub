use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::CreateType,
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    MyFederationData,
    services::{
        FederationServiceError,
        note::{ApubNoteModel, NoteWithApubModel},
        user::UserWithApubModel,
    },
};

use super::actor_check;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateActivity {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: CreateType,
    pub actor: ObjectId<UserWithApubModel>,
    #[serde(default)]
    pub to: Vec<Url>,
    #[serde(default)]
    pub cc: Vec<Url>,
    pub object: CreatableObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatableObject {
    Note(ApubNoteModel),
}

impl CreateActivity {
    pub fn from_note(note: ApubNoteModel) -> Self {
        let create_activity_id = {
            let note_url = note.id.inner().clone();
            Url::parse(format!("{note_url}/create").as_str()).unwrap()
        };
        Self {
            id: create_activity_id,
            kind: CreateType::Create,
            actor: note.attributed_to.clone(),
            to: note.to.clone(),
            cc: note.cc.clone(),
            object: CreatableObject::Note(note),
        }
    }
}

#[async_trait]
impl ActivityHandler for CreateActivity {
    type DataType = MyFederationData;
    type Error = FederationServiceError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // actor check
        match self.object {
            CreatableObject::Note(ref note) => {
                actor_check(self.actor(), note.attributed_to.inner())?;
                note.validate()?;
            }
        }

        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        match self.object {
            CreatableObject::Note(note) => {
                // save note
                let _ = NoteWithApubModel::from_json(note, data).await?;
            }
        }

        Ok(())
    }
}

#[test]
fn test_misskey_note_create() {
    let text = "
    {\"@context\":[\"https://www.w3.org/ns/activitystreams\",\"https://w3id.org/security/v1\",{\"Key\":\"sec:Key\",\"manuallyApprovesFollowers\":\"as:manuallyApprovesFollowers\",\"sensitive\":\"as:sensitive\",\"Hashtag\":\"as:Hashtag\",\"quoteUrl\":\"as:quoteUrl\",\"toot\":\"http://joinmastodon.org/ns#\",\"Emoji\":\"toot:Emoji\",\"featured\":\"toot:featured\",\"discoverable\":\"toot:discoverable\",\"schema\":\"http://schema.org#\",\"PropertyValue\":\"schema:PropertyValue\",\"value\":\"schema:value\",\"misskey\":\"https://misskey-hub.net/ns#\",\"_misskey_content\":\"misskey:_misskey_content\",\"_misskey_quote\":\"misskey:_misskey_quote\",\"_misskey_reaction\":\"misskey:_misskey_reaction\",\"_misskey_votes\":\"misskey:_misskey_votes\",\"_misskey_summary\":\"misskey:_misskey_summary\",\"isCat\":\"misskey:isCat\",\"vcard\":\"http://www.w3.org/2006/vcard/ns#\"}],\"id\":\"https://misskey.tinax.local/notes/a5fb07y3n8iq0003/activity\",\"actor\":\"https://misskey.tinax.local/users/9r70xhde0mav0001\",\"type\":\"Create\",\"published\":\"2025-03-16T14:37:11.355Z\",\"object\":{\"id\":\"https://misskey.tinax.local/notes/a5fb07y3n8iq0003\",\"type\":\"Note\",\"attributedTo\":\"https://misskey.tinax.local/users/9r70xhde0mav0001\",\"content\":\"<p><a href=\\\"https://lp.tinax.local/client/user/01JPFCHY08P7WM474CSYT85C2W\\\" class=\\\"u-url mention\\\">@admin@lp.tinax.local</a> testtest</p>\",\"published\":\"2025-03-16T14:37:11.355Z\",\"to\":[\"https://www.w3.org/ns/activitystreams#Public\"],\"cc\":[\"https://misskey.tinax.local/users/9r70xhde0mav0001/followers\",\"https://lp.tinax.local/user/01JPFCHY08P7WM474CSYT85C2W\"],\"inReplyTo\":null,\"attachment\":[],\"sensitive\":false,\"tag\":[{\"type\":\"Mention\",\"href\":\"https://lp.tinax.local/user/01JPFCHY08P7WM474CSYT85C2W\",\"name\":\"@admin@lp.tinax.local\"}]},\"to\":[\"https://www.w3.org/ns/activitystreams#Public\"],\"cc\":[\"https://misskey.tinax.local/users/9r70xhde0mav0001/followers\",\"https://lp.tinax.local/user/01JPFCHY08P7WM474CSYT85C2W\"]}
    ";
    let note: CreateActivity = serde_json::from_str(&text).unwrap();
    assert_eq!(
        note.id,
        Url::parse("https://misskey.tinax.local/notes/a5fb07y3n8iq0003/activity").unwrap()
    );
}
