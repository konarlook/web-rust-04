use crate::application::auth_service::AuthService;
use crate::application::blog_service::BlogService;
use crate::domain::post::{CreatePostReq, UpdatePostReq};
use crate::domain::user::{LoginRequest, RegisterRequest};
use crate::presentation::http::error::ApiError;
use crate::presentation::http::middleware::AuthenticatedUser;
use actix_web::{HttpResponse, web};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn register(
    body: web::Json<RegisterRequest>,
    auth: web::Data<AuthService>,
) -> Result<HttpResponse, ApiError> {
    let response = auth.register(body.into_inner()).await?;
    Ok(HttpResponse::Created().json(response))
}

pub async fn login(
    body: web::Json<LoginRequest>,
    auth: web::Data<AuthService>,
) -> Result<HttpResponse, ApiError> {
    let response = auth.login(body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn create_post(
    user: AuthenticatedUser,
    body: web::Json<CreatePostReq>,
    blog: web::Data<BlogService>,
) -> Result<HttpResponse, ApiError> {
    let post = blog.create(body.into_inner(), user.user_id).await?;
    Ok(HttpResponse::Ok().json(post))
}

pub async fn get_post(
    path: web::Path<i64>,
    blog: web::Data<BlogService>,
) -> Result<HttpResponse, ApiError> {
    let post = blog.get(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(post))
}

pub async fn list_posts(
    query: web::Query<Pagination>,
    blog: web::Data<BlogService>,
) -> Result<HttpResponse, ApiError> {
    let page = blog.list(query.limit, query.offset).await?;
    Ok(HttpResponse::Ok().json(page))
}

pub async fn update_post(
    user: AuthenticatedUser,
    path: web::Path<i64>,
    body: web::Json<UpdatePostReq>,
    blog: web::Data<BlogService>,
) -> Result<HttpResponse, ApiError> {
    let post = blog
        .update(path.into_inner(), body.into_inner(), user.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(post))
}

pub async fn delete_post(
    user: AuthenticatedUser,
    path: web::Path<i64>,
    blog: web::Data<BlogService>,
) -> Result<HttpResponse, ApiError> {
    blog.delete(path.into_inner(), user.user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
