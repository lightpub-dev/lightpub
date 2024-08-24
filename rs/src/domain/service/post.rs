use crate::{domain::model::post::PostId, holder, repository::interface::post::PostRepository};

pub struct PostService {
    post_repository: holder!(PostRepository),
}

impl PostService {
    pub async fn exists(&mut self, post_id: &PostId) -> Result<bool, anyhow::Error> {
        self.post_repository
            .find_by_id(post_id)
            .await
            .map(|post| post.is_some())
            .map_err(|e| e.into())
    }
}
