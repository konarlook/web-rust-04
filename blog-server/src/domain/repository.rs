use crate::domain::error::{PostError, UserError};
use crate::domain::post::{NewPost, Post, PostPage, UpdatePostReq};
use crate::domain::user::{NewUserRequest, User};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user_info: NewUserRequest) -> Result<User, UserError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, UserError>;
}

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: NewPost) -> Result<Post, PostError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, PostError>;
    async fn update(&self, id: i64, req: UpdatePostReq) -> Result<Post, PostError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<PostPage, PostError>;
    async fn delete(&self, id: i64) -> Result<(), PostError>;
}
