use thiserror::Error;

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
    #[error("internal server error")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("invalid token")]
    InvalidToken,
}

#[derive(Debug, Error)]
pub enum PostError {
    #[error("failed to create post, empty title")]
    EmptyTitle,
    #[error("post title must not exceed {0} characters")]
    TitleTooLong(usize),
    #[error("post {0} not found")]
    NotFound(i64),
    #[error("post belongs to another author")]
    Forbidden,
    #[error("storage failure")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}
