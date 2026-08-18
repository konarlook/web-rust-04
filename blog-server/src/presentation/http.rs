use crate::presentation::http::middleware::jwt_validator;
use actix_web::{guard, web};
use actix_web_httpauth::middleware::HttpAuthentication;

pub mod error;
pub mod handlers;
pub mod health;
pub mod middleware;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health::health_check))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(handlers::register))
                    .route("login", web::post().to(handlers::login)),
            )
            .service(
                web::scope("/posts")
                    .guard(guard::Get())
                    .route("", web::get().to(handlers::list_posts))
                    .route("/{id}", web::get().to(handlers::get_post)),
            )
            .service(
                web::scope("/posts")
                    .wrap(HttpAuthentication::bearer(jwt_validator))
                    .route("", web::post().to(handlers::create_post))
                    .route("/{id}", web::put().to(handlers::update_post))
                    .route("/{id}", web::delete().to(handlers::delete_post)),
            ),
    );
}
