use anyhow::{Context, bail};
use serde::Deserialize;

const MIN_JWT_SECRET_LEN: usize = 32;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub database_max_connect: u32,
    pub host: String,
    pub port: u16,
    pub grpc_port: u16,
    pub allowed_origins: Vec<String>,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let database_max_connect = std::env::var("DATABASE_MAX_CONNECT")
            .unwrap_or_else(|_| "10".into())
            .parse()?;

        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()?;

        let grpc_port = std::env::var("GRPC_PORT")
            .unwrap_or_else(|_| "50051".into())
            .parse()
            .context("GRPC_PORT must be a number")?;

        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".into())
            .split(',')
            .map(|s| s.trim().into())
            .collect();

        let jwt_secret = std::env::var("JWT_SECRET").context("JWT_SECRET must be set")?;
        if jwt_secret.len() < MIN_JWT_SECRET_LEN {
            bail!(
                "JWT_SECRET must be at least {MIN_JWT_SECRET_LEN} bytes long, got {}",
                jwt_secret.len()
            );
        }

        Ok(Self {
            database_url,
            database_max_connect,
            host,
            port,
            grpc_port,
            allowed_origins,
            jwt_secret,
        })
    }
}
