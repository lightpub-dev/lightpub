use async_trait::async_trait;

use crate::domain::model::post::{Post, PostId};

use super::RepositoryError;

#[async_trait]
pub trait PostRepository {
    async fn create(&mut self, post: &Post) -> Result<PostId, RepositoryError>;
    async fn delete(&mut self, post: &Post) -> Result<(), RepositoryError>;
    async fn find_by_id(&mut self, post_id: &PostId) -> Result<Option<Post>, RepositoryError>;
}
