use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("user {0} not found")]
    UserNotFound(Uuid),
    #[error("user with name {0} already exists")]
    UserAlreadyExists(String),
    #[error("incorrect login or password")]
    InvalidCredentials,
}

#[derive(Debug, Error)]
pub enum PostError {
    #[error("failed to create post, empty title")]
    EmptyPostTitle,
    #[error("post {0} not found")]
    PostNotFound(Uuid),
    #[error("forbidden")]
    Forbidden,
}
