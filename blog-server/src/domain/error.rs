use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("user {0} not found")]
    NotFound(String),
    #[error("user already exists")]
    AlreadyExists,
    #[error("incorrect login or password")]
    InvalidCredentials,
    #[error("storage failure")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
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
