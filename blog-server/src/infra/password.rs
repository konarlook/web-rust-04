use crate::domain::error::UserError;
use crate::domain::password::PasswordHasher as DomainHasher;
use argon2::password_hash::Error as PasswordHashError;
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use async_trait::async_trait;

const MEMORY_COST_KIB: u32 = 19 * 1024;
const TIME_COST: u32 = 2;
const PARALLELISM: u32 = 1;

pub struct ArgonHasher {
    params: Params,
}

impl ArgonHasher {
    pub fn new() -> Self {
        let params = Params::new(MEMORY_COST_KIB, TIME_COST, PARALLELISM, None)
            .expect("argon params must be valid");
        Self { params }
    }
}

impl Default for ArgonHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainHasher for ArgonHasher {
    async fn hash(&self, password: &str) -> Result<String, UserError> {
        let password = password.to_owned();
        let params = self.params.clone();

        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            argon
                .hash_password(password.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|e| UserError::Internal(Box::new(e)))
        })
        .await
        .map_err(|e| UserError::Internal(Box::new(e)))?
    }

    async fn verify(&self, password: &str, hash: &str) -> Result<bool, UserError> {
        let password = password.to_owned();
        let hash = hash.to_owned();

        tokio::task::spawn_blocking(move || {
            let parsed = PasswordHash::new(&hash).map_err(|e| UserError::Internal(Box::new(e)))?;

            let result = Argon2::default().verify_password(password.as_bytes(), &parsed);
            match result {
                Ok(()) => Ok(true),
                Err(PasswordHashError::Password) => Ok(false),
                Err(e) => {
                    tracing::error!(error = %e, "password hash is malformed");
                    Err(UserError::Internal(Box::new(e)))
                }
            }
        })
        .await
        .map_err(|e| UserError::Internal(Box::new(e)))?
    }
}
