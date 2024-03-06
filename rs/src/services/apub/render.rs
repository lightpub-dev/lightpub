use crate::models::{
    ApubPerson, ApubPersonBuilder, ApubRenderablePost, ApubRenderableUser, HasRemoteUri,
};
use crate::services::id::UserAttribute;
use crate::{models::ApubNote, models::ApubNoteBuilder, services::id::IDGetterService};

#[derive(Debug)]
pub struct ApubRendererService<'a> {
    id_getter: IDGetterService<'a>,
}

#[derive(Debug)]
pub enum ApubRendererServiceError {}

impl<'a> ApubRendererService<'a> {
    pub fn new(id_getter: IDGetterService<'a>) -> Self {
        Self { id_getter }
    }

    pub fn render_post(
        &self,
        post: &(impl ApubRenderablePost + HasRemoteUri),
    ) -> Result<ApubNote, ApubRendererServiceError> {
        let post_id = self.id_getter.get_post_id(post);
        let poster_id = self.id_getter.get_user_id(&post.poster());

        // let privacy = post.privacy();

        Ok(ApubNoteBuilder::default()
            .id(post_id)
            .attributed_to(poster_id)
            .content(post.content().unwrap())
            .to(vec![]) // TODO: to
            .cc(vec![]) // TODO: cc
            .published(post.created_at())
            .sensitive(false) // TODO: sensitive
            .build()
            .unwrap())
    }

    pub fn render_user(
        &self,
        user: &(impl ApubRenderableUser + HasRemoteUri),
    ) -> Result<ApubPerson, ApubRendererServiceError> {
        let user_id = self.id_getter.get_user_id(user);
        let inbox = self
            .id_getter
            .get_user_id_attr(user, UserAttribute::Inbox)
            .unwrap();
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

        let name = user.nickname();
        let preferred_username = user.username();

        Ok(ApubPersonBuilder::default()
            .id(user_id)
            .inbox(inbox)
            .outbox(outbox)
            .following(Some(following))
            .followers(Some(followers))
            .liked(Some(liked))
            .name(Some(name))
            .preferred_username(Some(preferred_username))
            .build()
            .unwrap())
    }
}
