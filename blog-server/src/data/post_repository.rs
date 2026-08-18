use crate::domain::error::PostError;
use crate::domain::post::{NewPost, Post, PostPage, UpdatePostReq};
use crate::domain::repository::PostRepository;
use async_trait::async_trait;
use sqlx::{Error, PgPool};

pub struct PgPostRepository {
    pool: PgPool,
}

impl PgPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PgPostRepository {
    async fn create(&self, post: NewPost) -> Result<Post, PostError> {
        sqlx::query_as!(
            Post,
            "
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, author_id, created_at, updated_at
            ",
            post.title,
            post.content,
            post.author_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(to_post_error)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, PostError> {
        sqlx::query_as!(
            Post,
            "
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts WHERE id = $1
            ",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(to_post_error)
    }

    async fn update(&self, id: i64, req: UpdatePostReq) -> Result<Post, PostError> {
        sqlx::query_as!(
            Post,
            "
            UPDATE posts
            SET title = COALESCE($2, title),
                content = COALESCE($3, content),
                updated_at = now()
            WHERE id = $1
            RETURNING id, title, content, author_id, created_at, updated_at
            ",
            id,
            req.title.as_deref(),
            req.content.as_deref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(to_post_error)?
        .ok_or(PostError::NotFound(id))
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<PostPage, PostError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at,
                   COUNT(*) OVER () AS "total!"
            FROM posts
            ORDER BY created_at DESC, id DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(to_post_error)?;

        let total = rows.first().map_or(0, |r| r.total);
        let posts = rows
            .into_iter()
            .map(|r| Post {
                id: r.id,
                title: r.title,
                content: r.content,
                author_id: r.author_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(PostPage { posts, total })
    }

    async fn delete(&self, id: i64) -> Result<(), PostError> {
        let result = sqlx::query!("DELETE FROM posts WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(to_post_error)?;
        if result.rows_affected() == 0 {
            return Err(PostError::NotFound(id));
        }
        Ok(())
    }
}

fn to_post_error(e: Error) -> PostError {
    PostError::Storage(Box::new(e))
}
