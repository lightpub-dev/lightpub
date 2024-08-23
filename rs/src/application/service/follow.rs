use crate::{
    domain::{
        factory::follow::{DefaultUserFollowFactory, UserFollowFactory},
        model::user::UserId,
    },
    holder,
    repository::interface::{uow::UnitOfWork, user::UserRepository},
};

pub struct FollowApplicationService {
    uow: holder!(UnitOfWork),
}

impl FollowApplicationService {
    pub async fn follow(
        &mut self,
        follower_id: UserId,
        followee_id: UserId,
    ) -> Result<(), anyhow::Error> {
        let mut follow_factory = DefaultUserFollowFactory::new();
        let follow = follow_factory.create(follower_id, followee_id);

        self.uow
            .repository_manager()
            .follow_repository()
            .follow(&follow)
            .await?;

        self.uow.commit().await?;

        Ok(())
    }

    pub async fn unfollow(
        &mut self,
        follower_id: UserId,
        followee_id: UserId,
    ) -> Result<(), anyhow::Error> {
        let follow = self
            .uow
            .repository_manager()
            .follow_repository()
            .find_by_user_id(&follower_id, &followee_id)
            .await?;

        // if follow is not found, do nothing
        if let Some(follow) = follow {
            self.uow
                .repository_manager()
                .follow_repository()
                .unfollow(&follow)
                .await?;
        }

        self.uow.commit().await?;

        Ok(())
    }
}
