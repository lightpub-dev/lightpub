use activitystreams::actor::Person;
use activitystreams::endpoint::EndpointProperties;
use activitystreams::ext::Ext;
use activitystreams::object::properties::ApObjectProperties;
use activitystreams::object::Note;

use crate::models::{ApubRenderablePost, ApubRenderableUser, HasRemoteUri};
use crate::services::id::IDGetterService;
use crate::services::id::UserAttribute;
use crate::utils::apub_key::{PublicKeyBuilder, PublicKeyExtension};
use activitystreams::ext::Extensible;

use anyhow::Result;

#[derive(Debug)]
pub struct ApubRendererService {
    id_getter: IDGetterService,
}

#[derive(Debug)]
pub enum ApubRendererServiceError {}

pub type ApubNote = Ext<Note, ApObjectProperties>;
pub type ApubPerson = Ext<
    Ext<Ext<Person, ApObjectProperties>, activitystreams::actor::properties::ApActorProperties>,
    PublicKeyExtension,
>;

impl ApubRendererService {
    pub fn new(id_getter: IDGetterService) -> Self {
        Self { id_getter }
    }

    pub fn render_post(
        &self,
        post: &(impl ApubRenderablePost + HasRemoteUri),
    ) -> Result<ApubNote, anyhow::Error> {
        let post_id = self.id_getter.get_post_id(post);
        let poster_id = self.id_getter.get_user_id(&post.poster());

        // let privacy = post.privacy();

        let mut note = Note::full();
        note.as_mut()
            .set_id(post_id)?
            .set_attributed_to_xsd_any_uri(poster_id)?
            .set_content_xsd_string(post.content().unwrap())?
            // .set_many_to_xsd_any_uris(vec![])?
            // .set_many_cc_xsd_any_uris(vec![])?
            .set_published(post.created_at_fixed_offset())?;

        Ok(note)
    }

    pub fn render_user(
        &self,
        user: &(impl ApubRenderableUser + HasRemoteUri),
    ) -> Result<ApubPerson> {
        let user_id = self.id_getter.get_user_id(user);
        let inbox = self
            .id_getter
            .get_user_id_attr(user, UserAttribute::Inbox)
            .unwrap();
        let shared_inbox: Option<String> = None; // TODO
        let outbox = self
            .id_getter
            .get_user_id_attr(user, UserAttribute::Outbox)
            .unwrap();
        let following = self
            .id_getter
            .get_user_id_attr(user, UserAttribute::Following)
            .unwrap();
        let followers = self
            .id_getter
            .get_user_id_attr(user, UserAttribute::Followers)
            .unwrap();
        let liked = self
            .id_getter
            .get_user_id_attr(user, UserAttribute::Liked)
            .unwrap();

        let mut person = Person::full();

        let public_key = {
            let pem = user.public_key().unwrap();
            let owner = user_id.clone();
            let key_id = self
                .id_getter
                .get_user_id_attr(user, UserAttribute::PublicKey)
                .unwrap();

            PublicKeyBuilder::default()
                .id(key_id)
                .owner(owner)
                .public_key_pem(pem)
                .build()
                .unwrap()
                .into_ext()
        };

        let name = user.nickname();
        let preferred_username = user.username();

        person.as_mut().set_id(user_id)?.set_name_xsd_string(name)?;
        person
            .extension
            .set_inbox(inbox)?
            .set_outbox(outbox)?
            .set_following(following)?
            .set_followers(followers)?
            .set_liked(liked)?
            .set_preferred_username(preferred_username)?
            .set_endpoints({
                let mut endpoints = EndpointProperties::default();
                if let Some(shared_inbox) = shared_inbox {
                    endpoints.set_shared_inbox(shared_inbox)?;
                }
                endpoints
            })?;
        let person = person.extend(public_key);

        Ok(person)
    }
}
