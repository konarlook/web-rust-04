use crate::domain::error::PostError;
use crate::domain::post::{CreatePostReq, NewPost, Post, PostPage, UpdatePostReq};
use crate::domain::repository::PostRepository;
use std::sync::Arc;

const DEFAULT_LIMIT: i64 = 10;
const MAX_LIMIT: i64 = 100;

pub struct BlogService {
    posts: Arc<dyn PostRepository>,
}

impl BlogService {
    pub fn new(posts: Arc<dyn PostRepository>) -> Self {
        Self { posts }
    }

    pub async fn create(&self, req: CreatePostReq, author_id: i64) -> Result<Post, PostError> {
        let new_post = NewPost::new(req, author_id)?;
        let post = self.posts.create(new_post).await?;

        tracing::info!(post_id = post.id, "post created");
        Ok(post)
    }

    pub async fn get(&self, id: i64) -> Result<Post, PostError> {
        self.posts
            .find_by_id(id)
            .await?
            .ok_or(PostError::NotFound(id))
    }

    pub async fn list(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<PostPage, PostError> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
        let offset = offset.unwrap_or(0).max(0);

        self.posts.list(limit, offset).await
    }

    pub async fn update(
        &self,
        id: i64,
        req: UpdatePostReq,
        user_id: i64,
    ) -> Result<Post, PostError> {
        self.ensure_author(id, user_id).await?;
        self.posts.update(id, req).await
    }

    pub async fn delete(&self, id: i64, user_id: i64) -> Result<(), PostError> {
        self.ensure_author(id, user_id).await?;
        self.posts.delete(id).await
    }

    async fn ensure_author(&self, id: i64, user_id: i64) -> Result<(), PostError> {
        let post = self
            .posts
            .find_by_id(id)
            .await?
            .ok_or(PostError::NotFound(id))?;
        if post.author_id != user_id {
            return Err(PostError::Forbidden);
        }
        Ok(())
    }
}
