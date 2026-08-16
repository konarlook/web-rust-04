use crate::domain::error::UserError;
use crate::domain::token::TokenIssuer;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

const TOKEN_TTL_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: i64,
}

pub struct JwtService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            validation: Validation::default(),
        }
    }

    pub fn generate_token(&self, user_id: i64, username: &str) -> Result<String, UserError> {
        let claims = Claims {
            user_id,
            username: username.to_owned(),
            exp: (Utc::now() + Duration::hours(TOKEN_TTL_HOURS)).timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding)
            .map_err(|e| UserError::Internal(Box::new(e)))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, UserError> {
        let raw_claim = decode::<Claims>(token, &self.decoding, &self.validation)
            .map_err(|_| UserError::InvalidToken)?;
        Ok(raw_claim.claims)
    }
}

impl TokenIssuer for JwtService {
    fn issue(&self, user_id: i64, username: &str) -> Result<String, UserError> {
        self.generate_token(user_id, username)
    }
}
