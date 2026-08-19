use crate::error::BlogClientError;
use crate::models::{AuthResponse, Post, PostPage};
use blog_proto::blog_service_client::BlogServiceClient;
use tonic::Request;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;

pub struct GrpcClient {
    client: BlogServiceClient<Channel>,
}

impl GrpcClient {
    pub async fn connect(endpoint: impl Into<String>) -> Result<Self, BlogClientError> {
        let client = BlogServiceClient::connect(endpoint.into()).await?;
        Ok(Self { client })
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .clone()
            .register(blog_proto::RegisterRequest {
                username: username.to_owned(),
                email: email.to_owned(),
                password: password.to_owned(),
            })
            .await?;
        response.into_inner().try_into()
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .clone()
            .login(blog_proto::LoginRequest {
                username: username.to_owned(),
                password: password.to_owned(),
            })
            .await?;
        response.into_inner().try_into()
    }

    pub async fn create_post(
        &self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let request = Self::with_token(
            blog_proto::CreatePostRequest {
                title: title.to_owned(),
                content: content.to_owned(),
            },
            token,
        )?;

        let response = self.client.clone().create_post(request).await?;
        response.into_inner().try_into()
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .clone()
            .get_post(blog_proto::GetPostRequest { id })
            .await?;
        response.into_inner().try_into()
    }

    pub async fn update_post(
        &self,
        token: &str,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, BlogClientError> {
        let request = Self::with_token(
            blog_proto::UpdatePostRequest {
                id,
                title: title.map(str::to_owned),
                content: content.map(str::to_owned),
            },
            token,
        )?;
        let response = self.client.clone().update_post(request).await?;
        response.into_inner().try_into()
    }

    pub async fn delete_post(&self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let request = Self::with_token(blog_proto::DeletePostRequest { id }, token)?;
        self.client.clone().delete_post(request).await?;
        Ok(())
    }

    pub async fn list_posts(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<PostPage, BlogClientError> {
        let response = self
            .client
            .clone()
            .list_posts(blog_proto::ListPostsRequest { limit, offset })
            .await?;
        response.into_inner().try_into()
    }

    fn with_token<T>(request: T, token: &str) -> Result<Request<T>, BlogClientError> {
        let mut request = Request::new(request);
        let value = format!("Bearer {token}");

        let value = MetadataValue::try_from(value)
            .map_err(|_| BlogClientError::InvalidRequest("token is not valid ASCII".into()))?;

        request.metadata_mut().insert("authorization", value);
        Ok(request)
    }
}
