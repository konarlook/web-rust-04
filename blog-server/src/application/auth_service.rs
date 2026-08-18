use crate::domain::error::UserError;
use crate::domain::password::PasswordHasher;
use crate::domain::repository::UserRepository;
use crate::domain::token::TokenIssuer;
use crate::domain::user::{LoginRequest, NewUserRequest, RegisterRequest, User};
use serde::Serialize;
use std::sync::Arc;

const DUMMY_PASSWORD: &str = "not correct password";

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

pub struct AuthService {
    users: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    token: Arc<dyn TokenIssuer>,
    dummy_hash: String,
}

impl AuthService {
    pub async fn new(
        users: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        token: Arc<dyn TokenIssuer>,
    ) -> Result<Self, UserError> {
        let dummy_hash = hasher.hash(DUMMY_PASSWORD).await?;
        Ok(Self {
            users,
            hasher,
            token,
            dummy_hash,
        })
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, UserError> {
        let password_hash = self.hasher.hash(&req.password).await?;

        let user = self
            .users
            .create(NewUserRequest {
                username: req.username,
                email: req.email,
                password_hash,
            })
            .await?;
        let token = self.token.issue(user.id, &user.username)?;

        tracing::info!(user_id = user.id, "user registered");
        Ok(AuthResponse { token, user })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, UserError> {
        let Some(user) = self.users.find_by_username(&req.username).await? else {
            self.hasher.verify(&req.password, &self.dummy_hash).await?;

            tracing::warn!(username = %req.username, "login attempt for unknown user");
            return Err(UserError::InvalidCredentials);
        };

        if !self
            .hasher
            .verify(&req.password, &user.password_hash)
            .await?
        {
            tracing::warn!(user_id = user.id, "login attempt with wrong password");
            return Err(UserError::InvalidCredentials);
        };

        tracing::info!(user_id = user.id, "user logged in");
        Ok(AuthResponse {
            token: self.token.issue(user.id, &user.username)?,
            user,
        })
    }
}
