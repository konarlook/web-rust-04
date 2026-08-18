use crate::domain::error::UserError;
use crate::domain::repository::UserRepository;
use crate::domain::user::{NewUserRequest, User};
use sqlx::{Error, PgPool};
use tonic::async_trait;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user_info: NewUserRequest) -> Result<User, UserError> {
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3) RETURNING *",
            user_info.username,
            user_info.email,
            user_info.password_hash,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(to_user_error)?;
        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, UserError> {
        let row = sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username,)
            .fetch_optional(&self.pool)
            .await
            .map_err(to_user_error)?;
        Ok(row)
    }
}

fn to_user_error(e: Error) -> UserError {
    if let Some(db) = e.as_database_error()
        && db.code().as_deref() == Some("23505")
    {
        return UserError::AlreadyExists;
    };
    UserError::Storage(Box::new(e))
}
