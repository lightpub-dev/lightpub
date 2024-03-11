use activitystreams::activity::properties::CreateProperties;
use activitystreams::activity::Create;
use activitystreams::actor::Person;
use activitystreams::endpoint::EndpointProperties;
use activitystreams::ext::Ext;
use activitystreams::object::properties::{ApObjectProperties, ObjectProperties};
use activitystreams::object::Note;
use derive_getters::Getters;
use uuid::Uuid;

use crate::models::{ApubRenderablePost, ApubRenderableUser, HasRemoteUri, PostPrivacy};
use crate::services::id::IDGetterService;
use crate::services::id::UserAttribute;
use crate::utils::apub_key::{PublicKeyBuilder, PublicKeyExtension};
use crate::utils::user::UserSpecifier;
use activitystreams::ext::Extensible;
use std::str::FromStr;

use anyhow::Result;

#[derive(Debug)]
pub struct ApubRendererService {
    id_getter: IDGetterService,
}

#[derive(Debug)]
pub enum ApubRendererServiceError {}

pub type ApubNoteCreate = Ext<Create, ApObjectProperties>;
pub type ApubNote = Ext<Note, ApObjectProperties>;
pub type ApubPerson = Ext<
    Ext<Ext<Person, ApObjectProperties>, activitystreams::actor::properties::ApActorProperties>,
    PublicKeyExtension,
>;

#[derive(Debug, Clone, Getters)]
pub struct RenderedNote {
    note: ApubNote,
    targeted_users: Vec<TargetedUser>,
}

#[derive(Debug, Clone, Getters)]
pub struct RenderedNoteCreate {
    note_create: ApubNoteCreate,
    targeted_users: Vec<TargetedUser>,
}

#[derive(Debug, Clone)]
pub enum TargetedUser {
    FollowerOf(UserSpecifier),
    Mentioned(UserSpecifier),
}

impl ApubRendererService {
    pub fn new(id_getter: IDGetterService) -> Self {
        Self { id_getter }
    }

    fn calculate_to_and_cc(
        &self,
        post: &impl ApubRenderablePost,
    ) -> Result<(Vec<String>, Vec<String>, Vec<TargetedUser>), anyhow::Error> {
        let privacy = post.privacy();
        let poster = &post.poster();
        let mut targets = vec![];

        let add_public = |v: &mut Vec<String>| {
            v.push(activitystreams::public().to_string());
        };
        let add_followers = |v: &mut Vec<String>| {
            let followers_uri = self
                .id_getter
                .get_user_id_attr(poster, UserAttribute::Followers)
                .expect("failed to get followers uri");
            v.push(followers_uri);
        };

        // https://socialhub.activitypub.rocks/t/visibility-to-cc-mapping/284/4
        let mut to = vec![];
        let mut cc = vec![];

        // TODO: handle mentions
        match privacy {
            PostPrivacy::Public => {
                add_public(&mut to);
                add_followers(&mut cc);
                targets.push(TargetedUser::FollowerOf(UserSpecifier::from_id(
                    Uuid::from_str(&poster.get_local_id()).unwrap(),
                )));
            }
            PostPrivacy::Unlisted => {
                add_public(&mut cc);
                add_followers(&mut cc);
                targets.push(TargetedUser::FollowerOf(UserSpecifier::from_id(
                    Uuid::from_str(&poster.get_local_id()).unwrap(),
                )));
            }
            PostPrivacy::Followers => {
                add_followers(&mut to);
                targets.push(TargetedUser::FollowerOf(UserSpecifier::from_id(
                    Uuid::from_str(&poster.get_local_id()).unwrap(),
                )));
            }
            PostPrivacy::Private => {}
        }

        Ok((to, cc, targets))
    }

    pub fn render_create_post(
        &self,
        post: &(impl ApubRenderablePost + HasRemoteUri),
    ) -> Result<RenderedNoteCreate, anyhow::Error> {
        let post = self.render_post(post)?;

        let post_to = post.note.as_ref().get_many_to_xsd_any_uris();
        let post_cc = post.note.as_ref().get_many_cc_xsd_any_uris();
        let post_bto = post.note.as_ref().get_many_bto_xsd_any_uris();
        let post_bcc = post.note.as_ref().get_many_bcc_xsd_any_uris();
        let post_poster = post
            .note
            .as_ref()
            .get_attributed_to_xsd_any_uri()
            .expect("attributedTo not set");

        let mut create = Create::full();
        {
            let m = AsMut::<ObjectProperties>::as_mut(&mut create);
            if let Some(to) = post_to {
                m.set_many_to_xsd_any_uris(to.map(|s| s.clone()).collect())?;
            }
            if let Some(cc) = post_cc {
                m.set_many_cc_xsd_any_uris(cc.map(|s| s.clone()).collect())?;
            }
            if let Some(bto) = post_bto {
                m.set_many_bto_xsd_any_uris(bto.map(|s| s.clone()).collect())?;
            }
            if let Some(bcc) = post_bcc {
                m.set_many_bcc_xsd_any_uris(bcc.map(|s| s.clone()).collect())?;
            }
        }
        AsMut::<CreateProperties>::as_mut(&mut create).set_actor_xsd_any_uri(post_poster.clone())?;

        Ok(RenderedNoteCreate {
            note_create: create,
            targeted_users: post.targeted_users,
        })
    }

    pub fn render_post(
        &self,
        post: &(impl ApubRenderablePost + HasRemoteUri),
    ) -> Result<RenderedNote, anyhow::Error> {
        let post_id = self.id_getter.get_post_id(post);
        let poster_id = self.id_getter.get_user_id(&post.poster());

        let (to, cc, targeted_users) = self.calculate_to_and_cc(post)?;

        let mut note = Note::full();
        note.as_mut()
            .set_id(post_id)?
            .set_attributed_to_xsd_any_uri(poster_id)?
            .set_content_xsd_string(post.content().unwrap())?
            .set_many_to_xsd_any_uris(to)?
            .set_many_cc_xsd_any_uris(cc)?
            .set_published(post.created_at_fixed_offset())?;

        Ok(RenderedNote {
            note,
            targeted_users,
        })
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
