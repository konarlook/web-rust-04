use crate::domain::error::PostError;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Post {
    pub fn new(req: CreatePostReq, author: Uuid) -> Result<Post, PostError> {
        if req.title.trim().is_empty() {
            return Err(PostError::EmptyPostTitle);
        }

        let now = Utc::now();
        Ok(Post {
            id: Uuid::now_v7(),
            title: req.title,
            content: req.content,
            author_id: author,
            created_at: now,
            updated_at: now,
        })
    }
}

pub struct CreatePostReq {
    pub title: String,
    pub content: String,
}

pub struct UpdatePostReq {
    pub title: String,
    pub content: String,
}
