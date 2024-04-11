use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::id::{IDGetterService, UserAttribute};
use lightpub_model::apub::{
    Activity, AnnounceActivity, AnnounceActivityBuilder, CreatableObject, CreateActivity,
    CreateActivityBuilder, DeleteActivity, IdOrObject, LikeActivity, NoteBuilder, Person,
    PersonBuilder, PublicKeyBuilder, TombstoneBuilder, UndoActivity, UndoableActivity, PUBLIC,
};
use lightpub_model::{
    ApubPostTargetComputable, ApubRenderablePost, ApubRenderableUser, HasRemoteUri,
    HasRemoteUriOnly, PostPrivacy, UserSpecifier,
};
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
    note: RenderedNoteObject,
    targeted_users: Vec<TargetedUser>,
}

#[derive(Debug, Clone)]
pub enum RenderedNoteObject {
    Create(CreatableObject),
    Announce(RepostInfo),
}

#[derive(Debug, Clone)]
pub struct RepostInfo {
    pub repost_id: String,
    pub reposter_uri: String,
    pub reposted_post_uri: String,
    pub repost_published_at: chrono::DateTime<chrono::Utc>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bto: Vec<String>,
    pub bcc: Vec<String>,
}

#[derive(Debug, Clone, Getters)]
pub struct RenderedNoteCreate {
    note_create: RenderedNoteCreateActivity,
    targeted_users: Vec<TargetedUser>,
}

#[derive(Debug, Clone)]
pub struct RenderedNoteDelete {
    pub note_delete: DeleteActivity,
    pub targeted_users: Vec<TargetedUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RenderedNoteCreateActivity {
    Create(CreateActivity),
    Announce(AnnounceActivity),
}

impl RenderedNoteCreateActivity {
    pub fn activity(self) -> Activity {
        use RenderedNoteCreateActivity::*;
        match self {
            Create(c) => Activity::Create(c),
            Announce(a) => Activity::Announce(a),
        }
    }
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

    pub fn render_post_reaction<A, P>(
        &self,
        internal_reaction_id: &str,
        actor: &A,
        post: &P,
        reaction: Option<impl Into<String>>,
        is_add: bool,
    ) -> Result<Activity, anyhow::Error>
    where
        A: HasRemoteUri,
        P: HasRemoteUri,
    {
        let activity_id = self
            .id_getter
            .get_reaction_id(internal_reaction_id, reaction.is_some());
        let actor_id = self.id_getter.get_user_id(actor);
        let post_id = self.id_getter.get_post_id(post);
        let content = reaction.map(|r| r.into());

        let like = LikeActivity {
            id: activity_id,
            actor: actor_id.clone(),
            object: post_id,
            content: content,
            published: None,
        };
        if is_add {
            Ok(Activity::Like(like))
        } else {
            Ok(Activity::Undo(UndoActivity {
                id: None,
                actor: actor_id,
                object: UndoableActivity::Like(like),
            }))
        }
    }

    pub fn calculate_post_involved_users<P: ApubPostTargetComputable>(
        &self,
        post: &P,
        include_poster: bool,
    ) -> Result<Vec<TargetedUser>, anyhow::Error> {
        let (_, _, mut targeted_users) = self.calculate_to_and_cc(post)?;
        if include_poster {
            targeted_users.push(TargetedUser::Mentioned(UserSpecifier::from_id(
                Uuid::from_str(&post.poster().get_local_id()).unwrap(),
            )));
        }
        Ok(targeted_users)
    }

    fn calculate_to_and_cc<P: ApubPostTargetComputable>(
        &self,
        post: &P,
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
            let uri = user.get_remote_uri();
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

        let activity = match post.note {
            RenderedNoteObject::Create(note) => match note {
                CreatableObject::Note(note) => {
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
                    RenderedNoteCreateActivity::Create(create)
                }
                CreatableObject::Tombstone(_) => {
                    return Err(anyhow::anyhow!("cannot create a tombstone"));
                }
            },
            RenderedNoteObject::Announce(rep) => RenderedNoteCreateActivity::Announce(
                AnnounceActivityBuilder::default()
                    .id(format!("{}/activity", rep.repost_id))
                    .actor(rep.reposter_uri)
                    .object(IdOrObject::Id(rep.reposted_post_uri))
                    .published(rep.repost_published_at)
                    .to(rep.to)
                    .cc(rep.cc)
                    .bto(Some(rep.bto))
                    .bcc(Some(rep.bcc))
                    .build()
                    .unwrap(),
            ),
        };

        Ok(RenderedNoteCreate {
            note_create: activity,
            targeted_users: post.targeted_users,
        })
    }

    pub fn render_delete_post<P, A>(
        &self,
        post: &P,
        author: &A,
    ) -> Result<RenderedNoteDelete, anyhow::Error>
    where
        P: ApubPostTargetComputable + HasRemoteUri,
        A: HasRemoteUri,
    {
        let (_, _, targeted_users) = self.calculate_to_and_cc(post)?;

        let post_id = self.id_getter.get_post_id(post);
        let actor = self.id_getter.get_user_id(author);

        let note_delete = DeleteActivity {
            id: None,
            actor,
            object: IdOrObject::Id(post_id),
        };

        Ok(RenderedNoteDelete {
            note_delete,
            targeted_users,
        })
    }

    pub fn render_post(
        &self,
        post: &(impl ApubRenderablePost + HasRemoteUri),
    ) -> Result<RenderedNote, anyhow::Error> {
        let post_id = self.id_getter.get_post_id(post);

        let poster_id = self.id_getter.get_user_id(&post.poster());

        let (to, cc, targeted_users) = self.calculate_to_and_cc(&post.as_target_computable())?;

        let (note, targeted_users) = {
            let note = if let Some(content) = post.content() {
                let note = match post.deleted_at_fixed_offset() {
                    // TODO: handle quoting posts
                    None => CreatableObject::Note(
                        NoteBuilder::default()
                            .id(post_id)
                            .attributed_to(poster_id)
                            .content(content)
                            .to(to)
                            .cc(cc)
                            .published(post.created_at_fixed_offset().to_utc())
                            .in_reply_to(post.reply_to_id().map(|id| Box::new(IdOrObject::Id(id))))
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
                RenderedNoteObject::Create(note)
            } else {
                RenderedNoteObject::Announce(RepostInfo {
                    repost_id: post_id,
                    reposter_uri: poster_id,
                    reposted_post_uri: post
                        .repost_of_id()
                        .expect("repost_of_id should not be None if content is None"),
                    repost_published_at: post.created_at_fixed_offset().to_utc(),
                    to,
                    cc,
                    bto: vec![],
                    bcc: vec![],
                })
            };
            (note, targeted_users)
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
