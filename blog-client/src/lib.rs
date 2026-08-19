use crate::error::BlogClientError;
use crate::grpc_client::GrpcClient;
use crate::http_client::HttpClient;
use crate::models::{AuthResponse, Post, PostPage};

pub mod error;
pub mod grpc_client;
pub mod http_client;
pub mod models;

#[derive(Debug, Clone)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

enum Backend {
    Http(HttpClient),
    Grpc(GrpcClient),
}

pub struct BlogClient {
    backend: Backend,
    token: Option<String>,
}

impl BlogClient {
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let backend = match transport {
            Transport::Http(url) => Backend::Http(HttpClient::new(url)?),
            Transport::Grpc(url) => Backend::Grpc(GrpcClient::connect(url).await?),
        };

        Ok(Self {
            backend,
            token: None,
        })
    }

    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = match &self.backend {
            Backend::Http(client) => client.register(username, email, password).await?,
            Backend::Grpc(client) => client.register(username, email, password).await?,
        };

        self.token = Some(response.token.clone());
        Ok(response)
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = match &self.backend {
            Backend::Http(client) => client.login(username, password).await?,
            Backend::Grpc(client) => client.login(username, password).await?,
        };
        self.token = Some(response.token.clone());
        Ok(response)
    }

    pub async fn create_post(&self, title: &str, content: &str) -> Result<Post, BlogClientError> {
        let token = self.require_token()?;

        match &self.backend {
            Backend::Http(client) => client.create_post(token, title, content).await,
            Backend::Grpc(client) => client.create_post(token, title, content).await,
        }
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        match &self.backend {
            Backend::Http(client) => client.get_post(id).await,
            Backend::Grpc(client) => client.get_post(id).await,
        }
    }

    pub async fn update_post(
        &self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, BlogClientError> {
        let token = self.require_token()?;

        match &self.backend {
            Backend::Http(client) => client.update_post(token, id, title, content).await,
            Backend::Grpc(client) => client.update_post(token, id, title, content).await,
        }
    }

    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        let token = self.require_token()?;

        match &self.backend {
            Backend::Http(client) => client.delete_post(token, id).await,
            Backend::Grpc(client) => client.delete_post(token, id).await,
        }
    }

    pub async fn list_posts(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<PostPage, BlogClientError> {
        match &self.backend {
            Backend::Http(client) => client.list_posts(limit, offset).await,
            Backend::Grpc(client) => client.list_posts(limit, offset).await,
        }
    }

    fn require_token(&self) -> Result<&str, BlogClientError> {
        self.token.as_deref().ok_or(BlogClientError::MissingToken)
    }
}
