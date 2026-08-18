use crate::domain::error::{PostError, UserError};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status).json(ErrorBody {
            error: &self.message,
        })
    }
}

impl From<UserError> for ApiError {
    fn from(err: UserError) -> Self {
        let status = match err {
            UserError::NotFound(_) => StatusCode::NOT_FOUND,
            UserError::AlreadyExists => StatusCode::CONFLICT,
            UserError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            UserError::InvalidToken => StatusCode::UNAUTHORIZED,
            UserError::Internal(_) | UserError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let message = if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = ?err, "internal error while handling request");
            "internal server error".to_owned()
        } else {
            err.to_string()
        };
        Self { status, message }
    }
}

impl From<PostError> for ApiError {
    fn from(err: PostError) -> Self {
        let status = match err {
            PostError::NotFound(_) => StatusCode::NOT_FOUND,
            PostError::Forbidden => StatusCode::FORBIDDEN,
            PostError::EmptyTitle | PostError::TitleTooLong(_) => StatusCode::BAD_REQUEST,
            PostError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let message = if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = ?err, "internal error while handling request");
            "internal server error".to_owned()
        } else {
            err.to_string()
        };

        Self { status, message }
    }
}
