pub mod domain;
pub mod handlers;
pub mod infra;

use crate::handlers::health::health_check;
use crate::infra::config::Config;
use crate::infra::database::{create_pool, run_migrations};
use crate::infra::logging::init_logging;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

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
            .into_utoipa_app()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .openapi_service(|api| {
                SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", api)
            })
            .into_app()
            .route("/health", web::get().to(health_check))
    })
    .bind(address)?
    .run()
    .await
}
