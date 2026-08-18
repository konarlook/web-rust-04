use crate::application::auth_service::AuthResponse;
use crate::domain::error::{PostError, UserError};
use crate::domain::post::{Post, PostPage};
use crate::domain::user::User;
use chrono::{DateTime, Utc};
use tonic::Status;

fn to_timestamp(dt: DateTime<Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

impl From<User> for blog_proto::User {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: Some(to_timestamp(user.created_at)),
        }
    }
}

impl From<Post> for blog_proto::Post {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: Some(to_timestamp(post.created_at)),
            updated_at: Some(to_timestamp(post.updated_at)),
        }
    }
}

impl From<AuthResponse> for blog_proto::AuthResponse {
    fn from(auth: AuthResponse) -> Self {
        Self {
            token: auth.token,
            user: Some(auth.user.into()),
        }
    }
}

impl From<PostPage> for blog_proto::ListPostsResponse {
    fn from(post_page: PostPage) -> Self {
        Self {
            posts: post_page.posts.into_iter().map(Into::into).collect(),
            total: post_page.total,
            limit: post_page.limit,
            offset: post_page.offset,
        }
    }
}

impl From<UserError> for Status {
    fn from(e: UserError) -> Self {
        match &e {
            UserError::NotFound(_) => Status::not_found(e.to_string()),
            UserError::AlreadyExists => Status::already_exists(e.to_string()),
            UserError::InvalidCredentials | UserError::InvalidToken => {
                Status::unauthenticated(e.to_string())
            }
            UserError::Storage(_) | UserError::Internal(_) => {
                tracing::error!(error = ?e, "internal error in gRPC handler");
                Status::internal("internal server error")
            }
        }
    }
}

impl From<PostError> for Status {
    fn from(e: PostError) -> Self {
        match &e {
            PostError::NotFound(_) => Status::not_found(e.to_string()),
            PostError::Forbidden => Status::permission_denied(e.to_string()),
            PostError::EmptyTitle | PostError::TitleTooLong(_) => {
                Status::invalid_argument(e.to_string())
            }
            PostError::Storage(_) => {
                tracing::error!(error = ?e, "internal error in gRPC handler");
                Status::internal("internal server error")
            }
        }
    }
}
