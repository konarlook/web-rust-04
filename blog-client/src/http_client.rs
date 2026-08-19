use crate::error::BlogClientError;
use crate::models::{AuthResponse, Post, PostPage};
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
struct ErrorBody {
    error: String,
}

pub struct HttpClient {
    client: Client,
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, BlogClientError> {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()?;
        Ok(Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .post(self.url("/api/auth/register"))
            .json(&RegisterBody {
                username,
                email,
                password,
            })
            .send()
            .await?;
        parse_body(response).await
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .post(self.url("/api/auth/login"))
            .json(&LoginBody { username, password })
            .send()
            .await?;
        parse_body(response).await
    }

    pub async fn create_post(
        &self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .post(self.url("api/posts"))
            .bearer_auth(token)
            .json(&CreatePostBody { title, content })
            .send()
            .await?;
        parse_body(response).await
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .get(self.url(&format!("/api/posts/{id}")))
            .send()
            .await?;
        parse_body(response).await
    }

    pub async fn update_post(
        &self,
        token: &str,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .put(self.url(&format!("/api/posts/{id}")))
            .bearer_auth(token)
            .json(&UpdatePostBody { title, content })
            .send()
            .await?;
        parse_body(response).await
    }

    pub async fn delete_post(&self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let response = self
            .client
            .delete(self.url(&format!("/api/posts/{id}")))
            .bearer_auth(token)
            .send()
            .await?;

        ensure_success(response).await?;
        Ok(())
    }

    pub async fn list_posts(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<PostPage, BlogClientError> {
        let mut request = self.client.get(self.url("/api/posts"));

        if let Some(limit) = limit {
            request = request.query(&[("limit", limit)]);
        }
        if let Some(offset) = offset {
            request = request.query(&[("offset", offset)]);
        }

        let response = request.send().await?;
        parse_body(response).await
    }
}

#[derive(Serialize)]
struct RegisterBody<'a> {
    username: &'a str,
    email: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct LoginBody<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct CreatePostBody<'a> {
    title: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct UpdatePostBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
}

async fn ensure_success(response: Response) -> Result<Response, BlogClientError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let message = response
        .text()
        .await
        .ok()
        .and_then(|body| serde_json::from_str::<ErrorBody>(&body).ok())
        .map(|b| b.error)
        .unwrap_or_else(|| status.to_string());

    Err(match status {
        StatusCode::NOT_FOUND => BlogClientError::NotFound,
        StatusCode::UNAUTHORIZED => BlogClientError::Unauthorized,
        _ if status.is_client_error() => BlogClientError::InvalidRequest(message),
        _ => BlogClientError::Server(message),
    })
}

async fn parse_body<T: DeserializeOwned>(response: Response) -> Result<T, BlogClientError> {
    let response = ensure_success(response).await?;
    Ok(response.json::<T>().await?)
}
