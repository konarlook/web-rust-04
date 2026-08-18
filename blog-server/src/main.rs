pub mod application;
pub mod data;
pub mod domain;
pub mod infra;
pub mod presentation;

use crate::application::auth_service::AuthService;
use crate::application::blog_service::BlogService;
use crate::data::post_repository::PgPostRepository;
use crate::data::user_repository::PgUserRepository;
use crate::infra::config::Config;
use crate::infra::database::{create_pool, run_migrations};
use crate::infra::jwt::JwtService;
use crate::infra::logging::init_logging;
use crate::infra::password::ArgonHasher;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use std::sync::Arc;

const CORS_MAX_AGE_SECS: usize = 3600;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let cfg = Config::from_env()?;

    let pool = create_pool(&cfg).await?;
    run_migrations(&pool).await?;

    let jwt = Arc::new(JwtService::new(&cfg.jwt_secret));
    let hasher = Arc::new(ArgonHasher::new());

    let auth_service = Arc::new(
        AuthService::new(
            Arc::new(PgUserRepository::new(pool.clone())),
            hasher,
            jwt.clone(),
        )
        .await?,
    );
    let blog_service = Arc::new(BlogService::new(Arc::new(PgPostRepository::new(pool))));

    let auth_data = web::Data::from(auth_service.clone());
    let blog_data = web::Data::from(blog_service.clone());
    let jwt_data = web::Data::from(jwt.clone());
    let origins = cfg.allowed_origins.clone();

    let http_addr = format!("{}:{}", cfg.host, cfg.port);
    tracing::info!(address = %http_addr, "starting HTTP server");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(build_cors(&origins))
            .app_data(auth_data.clone())
            .app_data(blog_data.clone())
            .app_data(jwt_data.clone())
            .configure(presentation::http::configure)
    })
    .bind(&http_addr)?
    .run()
    .await?;

    Ok(())
}

fn build_cors(origins: &[String]) -> Cors {
    let cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_any_header()
        .max_age(CORS_MAX_AGE_SECS);

    if origins.iter().any(|origin| origin == "*") {
        return cors.allow_any_origin();
    }

    origins
        .iter()
        .fold(cors, |cors, origin| cors.allowed_origin(origin))
}
