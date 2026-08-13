use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub database_max_connect: u32,
    pub host: String,
    pub port: u16,
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

        Ok(Self {
            database_url,
            database_max_connect,
            host,
            port,
        })
    }
}
