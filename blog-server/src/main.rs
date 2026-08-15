pub mod domain;
pub mod infra;
pub mod presentation;
pub mod data;

use crate::infra::config::Config;
use crate::infra::database::{create_pool, run_migrations};
use crate::infra::logging::init_logging;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let cfg = Config::from_env().expect("invalid config");

    let pool = create_pool(&cfg)
        .await
        .expect("Failed to create database pool");
    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    let address = format!("{}:{}", cfg.host, cfg.port);

    tracing::info!("Starting server at {}", &address);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(pool.clone())
            .app_data(web::Data::new(cfg.clone()))
    })
    .bind(address)?
    .run()
    .await
}
