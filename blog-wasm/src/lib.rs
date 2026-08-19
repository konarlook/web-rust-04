pub mod models;
pub mod storage;

use crate::models::{
    AuthResponse, CreatePostBody, LoginBody, Post, PostPage, RegisterBody, UpdatePostBody,
};
use gloo_net::http::{Request, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

const DEFAULT_API_BASE: &str = "http://localhost:8080";

#[wasm_bindgen]
pub struct BlogApp {
    api_base: String,
    token: Option<String>,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new(api_base: Option<String>) -> Self {
        Self {
            api_base: api_base.unwrap_or_else(|| DEFAULT_API_BASE.to_owned()),
            token: storage::load_token(),
        }
    }

    #[wasm_bindgen(js_name = isAuthenticated)]
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub fn logout(&mut self) {
        self.token = None;
        storage::clear_token();
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        let response = Request::post(&self.url("/api/auth/register"))
            .json(&RegisterBody {
                username: &username,
                email: &email,
                password: &password,
            })
            .map_err(to_js_error)?
            .send()
            .await
            .map_err(to_js_error)?;
        let auth: AuthResponse = read_json(response).await?;
        to_js(&auth)
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<JsValue, JsValue> {
        let response = Request::post(&self.url("/api/auth/login"))
            .json(&LoginBody {
                username: &username,
                password: &password,
            })
            .map_err(to_js_error)?
            .send()
            .await
            .map_err(to_js_error)?;
        let auth: AuthResponse = read_json(response).await?;
        self.remember(&auth.token);
        to_js(&auth)
    }

    #[wasm_bindgen(js_name = loadPosts)]
    pub async fn load_posts(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<JsValue, JsValue> {
        let mut url = self.url("/api/posts");
        let mut params = Vec::new();
        if let Some(limit) = limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(offset) = offset {
            params.push(format!("offset={offset}"));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = Request::get(&url).send().await.map_err(to_js_error)?;
        let page: PostPage = read_json(response).await?;
        to_js(&page)
    }

    #[wasm_bindgen(js_name = createPost)]
    pub async fn create_post(&self, title: String, content: String) -> Result<JsValue, JsValue> {
        let token = self.require_token()?;
        let response = Request::post(&self.url("/api/posts"))
            .header("Authorization", &format!("Bearer {token}"))
            .json(&CreatePostBody {
                title: &title,
                content: &content,
            })
            .map_err(to_js_error)?
            .send()
            .await
            .map_err(to_js_error)?;
        let post: Post = read_json(response).await?;
        to_js(&post)
    }

    #[wasm_bindgen(js_name = updatePost)]
    pub async fn update_post(
        &self,
        id: i64,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let token = self.require_token()?;

        let response = Request::put(&self.url(&format!("/api/posts/{id}")))
            .header("Authorization", &format!("Bearer {token}"))
            .json(&UpdatePostBody {
                title: title.as_deref(),
                content: content.as_deref(),
            })
            .map_err(to_js_error)?
            .send()
            .await
            .map_err(to_js_error)?;

        let post: Post = read_json(response).await?;
        to_js(&post)
    }

    #[wasm_bindgen(js_name = deletePost)]
    pub async fn delete_post(&self, id: i64) -> Result<JsValue, JsValue> {
        let token = self.require_token()?;

        let response = Request::delete(&self.url(&format!("/api/posts/{id}")))
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await
            .map_err(to_js_error)?;

        ensure_ok(&response).await?;
        Ok(JsValue::NULL)
    }
}

impl BlogApp {
    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.api_base, path.trim_start_matches('/'))
    }

    fn remember(&mut self, token: &str) {
        self.token = Some(token.to_owned());
        storage::save_token(token);
    }

    fn require_token(&self) -> Result<&str, JsValue> {
        self.token
            .as_deref()
            .ok_or_else(|| to_js_error("authentication required: log in first"))
    }
}

fn to_js_error(err: impl ToString) -> JsValue {
    js_sys::Error::new(&err.to_string()).into()
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(to_js_error)
}

fn http_error(status: u16, message: &str) -> JsValue {
    let error = js_sys::Error::new(message);
    let _ = js_sys::Reflect::set(error.as_ref(), &"status".into(), &JsValue::from(status));
    error.into()
}

async fn ensure_ok(response: &Response) -> Result<(), JsValue> {
    if response.ok() {
        return Ok(());
    }

    let status = response.status();

    let detail = response
        .text()
        .await
        .ok()
        .and_then(|body| serde_json::from_str::<serde_json::Value>(&body).ok())
        .and_then(|json| json["error"].as_str().map(str::to_owned))
        .unwrap_or_else(|| response.status_text());

    Err(http_error(status, &detail))
}

async fn read_json<T: DeserializeOwned>(response: Response) -> Result<T, JsValue> {
    ensure_ok(&response).await?;
    response.json::<T>().await.map_err(to_js_error)
}
