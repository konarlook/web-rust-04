use crate::domain::error::PostError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const MAX_TITLE_LEN: usize = 256;

#[derive(Debug, Serialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: Option<String>,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewPost {
    pub title: String,
    pub content: Option<String>,
    pub author_id: i64,
}

impl NewPost {
    pub fn new(req: CreatePostReq, author_id: i64) -> Result<Self, PostError> {
        let title = req.title.trim();
        if title.is_empty() {
            return Err(PostError::EmptyTitle);
        }
        if title.chars().count() > MAX_TITLE_LEN {
            return Err(PostError::TitleTooLong(MAX_TITLE_LEN));
        }

        Ok(Self {
            title: title.to_owned(),
            content: Option::from(req.content),
            author_id,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePostReq {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostReq {
    pub title: Option<String>,
    pub content: Option<String>,
}
