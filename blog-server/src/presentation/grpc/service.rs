use crate::application::auth_service::AuthService;
use crate::application::blog_service::BlogService;
use crate::domain::post::{CreatePostReq, UpdatePostReq};
use crate::domain::user::{LoginRequest, RegisterRequest};
use crate::infra::jwt::JwtService;
use blog_proto::blog_service_server::BlogService as BlogServiceRpc;

use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct BlogGrpcService {
    auth: Arc<AuthService>,
    blog: Arc<BlogService>,
    jwt: Arc<JwtService>,
}

impl BlogGrpcService {
    pub fn new(auth: Arc<AuthService>, blog: Arc<BlogService>, jwt: Arc<JwtService>) -> Self {
        Self { auth, blog, jwt }
    }

    fn authenticate<T>(&self, request: &Request<T>) -> Result<i64, Status> {
        let header = request
            .metadata()
            .get("authorization")
            .ok_or_else(|| Status::unauthenticated("missing authorization metadata"))?;

        let value = header
            .to_str()
            .map_err(|_| Status::unauthenticated("authorization metadata is not valid ASCII"))?;

        let token = value
            .split_once(' ')
            .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("bearer"))
            .map(|(_, token)| token)
            .ok_or_else(|| Status::unauthenticated("expected \"Bearer token format\""))?;

        let claim = self
            .jwt
            .verify_token(token)
            .map_err(|_| Status::unauthenticated("invalid or expired token"))?;

        Ok(claim.user_id)
    }
}

#[tonic::async_trait]
impl BlogServiceRpc for BlogGrpcService {
    async fn register(
        &self,
        request: Request<blog_proto::RegisterRequest>,
    ) -> Result<Response<blog_proto::AuthResponse>, Status> {
        let req = request.into_inner();
        let result = self
            .auth
            .register(RegisterRequest {
                username: req.username,
                email: req.email,
                password: req.password,
            })
            .await?;
        Ok(Response::new(result.into()))
    }

    async fn login(
        &self,
        request: Request<blog_proto::LoginRequest>,
    ) -> Result<Response<blog_proto::AuthResponse>, Status> {
        let req = request.into_inner();
        let result = self
            .auth
            .login(LoginRequest {
                username: req.username,
                password: req.password,
            })
            .await?;
        Ok(Response::new(result.into()))
    }

    async fn create_post(
        &self,
        request: Request<blog_proto::CreatePostRequest>,
    ) -> Result<Response<blog_proto::PostResponse>, Status> {
        let author_id = self.authenticate(&request)?;
        let req = request.into_inner();

        let post = self
            .blog
            .create(
                CreatePostReq {
                    title: req.title,
                    content: req.content,
                },
                author_id,
            )
            .await?;
        Ok(Response::new(blog_proto::PostResponse {
            post: Some(post.into()),
        }))
    }

    async fn get_post(
        &self,
        request: Request<blog_proto::GetPostRequest>,
    ) -> Result<Response<blog_proto::PostResponse>, Status> {
        let post = self.blog.get(request.into_inner().id).await?;

        Ok(Response::new(blog_proto::PostResponse {
            post: Some(post.into()),
        }))
    }

    async fn update_post(
        &self,
        request: Request<blog_proto::UpdatePostRequest>,
    ) -> Result<Response<blog_proto::PostResponse>, Status> {
        let user_id = self.authenticate(&request)?;
        let req = request.into_inner();

        let post = self
            .blog
            .update(
                req.id,
                UpdatePostReq {
                    title: req.title,
                    content: req.content,
                },
                user_id,
            )
            .await?;
        Ok(Response::new(blog_proto::PostResponse {
            post: Some(post.into()),
        }))
    }

    async fn delete_post(
        &self,
        request: Request<blog_proto::DeletePostRequest>,
    ) -> Result<Response<blog_proto::DeletePostResponse>, Status> {
        let user_id = self.authenticate(&request)?;
        self.blog.delete(request.into_inner().id, user_id).await?;

        Ok(Response::new(blog_proto::DeletePostResponse {
            success: true,
        }))
    }

    async fn list_posts(
        &self,
        request: Request<blog_proto::ListPostsRequest>,
    ) -> Result<Response<blog_proto::ListPostsResponse>, Status> {
        let req = request.into_inner();
        let page = self.blog.list(req.limit, req.offset).await?;

        Ok(Response::new(page.into()))
    }
}
