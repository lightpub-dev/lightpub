use derive_more::Constructor;

use crate::{
    domain::{
        factory::follow::{DefaultUserFollowFactory, UserFollowFactory},
        model::user::UserId,
    },
    holder,
    repository::interface::uow::UnitOfWork,
};

#[derive(Constructor)]
pub struct FollowApplicationService {
    uow: holder!(UnitOfWork),
}

impl FollowApplicationService {
    pub async fn follow(
        &mut self,
        follower_id: &str,
        followee_id: &str,
    ) -> Result<(), anyhow::Error> {
        let follower_id = UserId::from_str(follower_id).unwrap();
        let followee_id = UserId::from_str(followee_id).unwrap();

        let mut follow_factory = DefaultUserFollowFactory::new();
        let mut follow = follow_factory.create(follower_id, followee_id);

        {
            let mut repo = self.uow.repository_manager().await?;

            repo.follow_repository()
                .create_if_not_exists(&mut follow)
                .await?;
        }

        self.uow.commit().await?;

        Ok(())
    }

    pub async fn unfollow(
        &mut self,
        follower_id: &str,
        followee_id: &str,
    ) -> Result<(), anyhow::Error> {
        let follower_id = UserId::from_str(follower_id).unwrap();
        let followee_id = UserId::from_str(followee_id).unwrap();

        {
            let mut repo = self.uow.repository_manager().await?;
            let follow = repo
                .follow_repository()
                .find_by_user_id(&follower_id, &followee_id)
                .await?;

            // if follow is not found, do nothing
            if let Some(follow) = follow {
                repo.follow_repository().delete_if_exists(&follow).await?;
            }
        }

        self.uow.commit().await?;

        Ok(())
    }
}
