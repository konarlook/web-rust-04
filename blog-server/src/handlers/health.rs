use actix_web::{HttpResponse, Result};

pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json("ok"))
}
