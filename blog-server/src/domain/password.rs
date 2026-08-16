use crate::domain::error::UserError;
use async_trait::async_trait;

#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, password: &str) -> Result<String, UserError>;
    async fn verify(&self, password: &str, hash: &str) -> Result<bool, UserError>;
}
