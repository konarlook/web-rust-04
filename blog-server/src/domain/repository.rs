use crate::domain::error::UserError;
use crate::domain::user::{NewUserRequest, User};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user_info: NewUserRequest) -> Result<User, UserError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, UserError>;
}
