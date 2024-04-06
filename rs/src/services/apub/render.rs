use derive_getters::Getters;
use uuid::Uuid;

use crate::models::apub::{
    CreatableObject, CreateActivity, CreateActivityBuilder, IdOrObject, NoteBuilder, Person,
    PersonBuilder, PublicKeyBuilder, TombstoneBuilder, PUBLIC,
};
use crate::models::{ApubRenderablePost, ApubRenderableUser, HasRemoteUri, PostPrivacy};
use crate::services::id::IDGetterService;
use crate::services::id::UserAttribute;
use crate::utils::user::UserSpecifier;
use std::str::FromStr;

use anyhow::Result;

#[derive(Debug)]
pub struct ApubRendererService {
    id_getter: IDGetterService,
}

#[derive(Debug)]
pub enum ApubRendererServiceError {}

#[derive(Debug, Clone, Getters)]
pub struct RenderedNote {
    note: CreatableObject,
    targeted_users: Vec<TargetedUser>,
}

#[derive(Debug, Clone, Getters)]
pub struct RenderedNoteCreate {
    note_create: CreateActivity,
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
            v.push(PUBLIC.to_owned());
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

        // add mentioned users
        for user in post.mentioned() {
            let uri = user.uri();
            if let Some(uri) = uri {
                to.push(uri.clone());
                targets.push(TargetedUser::Mentioned(UserSpecifier::from_url(uri)));
            }
        }

        Ok((to, cc, targets))
    }

    pub fn render_create_post(
        &self,
        post: &(impl ApubRenderablePost + HasRemoteUri),
    ) -> Result<RenderedNoteCreate, anyhow::Error> {
        let post = self.render_post(post)?;

        let note = match post.note {
            CreatableObject::Note(note) => note,
            CreatableObject::Tombstone(_) => {
                return Err(anyhow::anyhow!("cannot create a tombstone"));
            }
        };

        let create_id = format!("{}/activity", note.id); // FIXME: use IDGetterService
        let post_to = &note.to;
        let post_cc = &note.cc;
        let post_bto = &note.bto;
        let post_bcc = &note.bcc;
        let post_poster = &note.attributed_to;

        let create = CreateActivityBuilder::default()
            .actor(post_poster.clone())
            .id(create_id.clone())
            .object(IdOrObject::Object(CreatableObject::Note(note.clone())))
            .to(post_to.clone())
            .cc(post_cc.clone())
            .bto(post_bto.clone())
            .bcc(post_bcc.clone())
            .build()
            .unwrap();

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

        let note = match post.deleted_at_fixed_offset() {
            None => CreatableObject::Note(
                NoteBuilder::default()
                    .id(post_id)
                    .attributed_to(poster_id)
                    .content(post.content().unwrap())
                    .to(to)
                    .cc(cc)
                    .published(post.created_at_fixed_offset().to_utc())
                    .build()
                    .unwrap(),
            ),
            Some(deleted_at) => CreatableObject::Tombstone(
                TombstoneBuilder::default()
                    .id(post_id)
                    .deleted(deleted_at.to_utc())
                    .published(post.created_at_fixed_offset().to_utc())
                    .build()
                    .unwrap(),
            ),
        };

        Ok(RenderedNote {
            note,
            targeted_users,
        })
    }

    pub fn render_user(&self, user: &(impl ApubRenderableUser + HasRemoteUri)) -> Result<Person> {
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
        };

        let name = user.nickname();
        let preferred_username = user.username();

        let person = PersonBuilder::default()
            .id(user_id)
            .name(name)
            .inbox(inbox)
            .outbox(outbox)
            .following(following.into())
            .followers(followers.into())
            .liked(liked.into())
            .preferred_username(preferred_username)
            .shared_inbox(shared_inbox.into())
            .public_key(public_key.into())
            .summary(Some(user.bio()))
            .build()
            .unwrap();

        Ok(person)
    }
}
