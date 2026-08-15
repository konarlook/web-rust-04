use crate::domain::error::UserError;
use crate::domain::user::{NewUserRequest, User};
use tonic::async_trait;

#[async_trait]
pub trait UserRepository {
    async fn create(&self, user_info: NewUserRequest) -> Result<User, UserError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, UserError>;
}

pub trait PasswordHasher {
    fn hash_password(&self, password: &str) -> Result<String, UserError>;
}
