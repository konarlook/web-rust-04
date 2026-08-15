use actix_web::HttpResponse;
use actix_web::web::scope;


async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json("ok")
}
