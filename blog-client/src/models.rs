use crate::error::BlogClientError;
use blog_proto::PostResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<blog_proto::User> for User {
    type Error = BlogClientError;

    fn try_from(user: blog_proto::User) -> Result<Self, Self::Error> {
        Ok(Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: to_datetime(user.created_at, "user.created_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<blog_proto::Post> for Post {
    type Error = BlogClientError;

    fn try_from(post: blog_proto::Post) -> Result<Self, Self::Error> {
        Ok(Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: to_datetime(post.created_at, "post.created_at")?,
            updated_at: to_datetime(post.updated_at, "post.updated_at")?,
        })
    }
}

impl TryFrom<blog_proto::PostResponse> for Post {
    type Error = BlogClientError;

    fn try_from(resp: PostResponse) -> Result<Self, Self::Error> {
        resp.post
            .ok_or(BlogClientError::MalformedResponse("post_response.post"))?
            .try_into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

impl TryFrom<blog_proto::AuthResponse> for AuthResponse {
    type Error = BlogClientError;

    fn try_from(response: blog_proto::AuthResponse) -> Result<Self, Self::Error> {
        let user = response
            .user
            .ok_or(BlogClientError::MalformedResponse("auth_response.user"))?;

        Ok(Self {
            token: response.token,
            user: user.try_into()?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostPage {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl TryFrom<blog_proto::ListPostsResponse> for PostPage {
    type Error = BlogClientError;

    fn try_from(response: blog_proto::ListPostsResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            posts: response
                .posts
                .into_iter()
                .map(Post::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            total: response.total,
            limit: response.limit,
            offset: response.offset,
        })
    }
}

fn to_datetime(
    ts: Option<prost_types::Timestamp>,
    field: &'static str,
) -> Result<DateTime<Utc>, BlogClientError> {
    let ts = ts.ok_or(BlogClientError::MalformedResponse(field))?;

    let nanos = u32::try_from(ts.nanos).map_err(|_| BlogClientError::MalformedResponse(field))?;

    DateTime::from_timestamp(ts.seconds, nanos).ok_or(BlogClientError::MalformedResponse(field))
}
