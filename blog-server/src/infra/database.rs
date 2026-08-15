use crate::infra::config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, migrate};

pub async fn create_pool(cfg: &Config) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(cfg.database_max_connect)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&cfg.database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    migrate!("./migrations").run(pool).await?;
    tracing::info!("Migrations completed");
    Ok(())
}
