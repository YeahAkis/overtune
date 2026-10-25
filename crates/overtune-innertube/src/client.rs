use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, ORIGIN, REFERER, USER_AGENT};
use serde_json::{Map, Value};
use tokio::sync::Mutex;

use crate::context::{ClientContext, DEFAULT_VISITOR_DATA};
use crate::error::{Error, Result};

const API_KEY: &str = "AIzaSyC9XL3ZjWddXya6X74dJoCTL-WEYFDNX30";
const MUSIC_ORIGIN: &str = "https://music.youtube.com";
const MUSIC_BASE: &str = "https://music.youtube.com/youtubei/v1";
pub(crate) const VISITOR_ID: &str = "https://youtubei.googleapis.com/youtubei/v1/visitor_id";

#[derive(Debug, Clone)]
pub struct Innertube {
    http: Client,
    hl: String,
    gl: String,
    visitor: Arc<Mutex<Option<String>>>,
}

impl Innertube {
    pub fn new(hl: &str, gl: &str) -> Innertube {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .gzip(true)
            .brotli(true)
            .build()
            .expect("http client");
        Innertube {
            http,
            hl: hl.to_owned(),
            gl: gl.to_owned(),
            visitor: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn music_url(path: &str) -> String {
        format!("{MUSIC_BASE}/{path}")
    }

    pub async fn visitor_data(&self) -> Result<String> {
        if let Some(v) = self.visitor.lock().await.clone() {
            return Ok(v);
        }
        self.refresh_visitor_data().await
    }

    pub async fn refresh_visitor_data(&self) -> Result<String> {
        let mut guard = self.visitor.lock().await;
        let response = self
            .send(
                VISITOR_ID,
                &ClientContext::VISIONOS,
                DEFAULT_VISITOR_DATA,
                Map::new(),
            )
            .await?;
        let visitor = response
            .pointer("/responseContext/visitorData")
            .and_then(Value::as_str)
            .ok_or(Error::Parse("visitor data missing"))?
            .to_owned();
        *guard = Some(visitor.clone());
        Ok(visitor)
    }

    async fn current_visitor(&self) -> String {
        self.visitor
            .lock()
            .await
            .clone()
            .unwrap_or_else(|| DEFAULT_VISITOR_DATA.to_owned())
    }

    pub(crate) async fn post(
        &self,
        url: &str,
        ctx: &ClientContext,
        body: Map<String, Value>,
    ) -> Result<Value> {
        let visitor = self.current_visitor().await;
        self.send_with_query(url, ctx, &visitor, body, &[]).await
    }

    pub(crate) async fn post_with_query(
        &self,
        url: &str,
        ctx: &ClientContext,
        body: Map<String, Value>,
        query: &[(&str, &str)],
    ) -> Result<Value> {
        let visitor = self.current_visitor().await;
        self.send_with_query(url, ctx, &visitor, body, query).await
    }

    async fn send(
        &self,
        url: &str,
        ctx: &ClientContext,
        visitor: &str,
        body: Map<String, Value>,
    ) -> Result<Value> {
        self.send_with_query(url, ctx, visitor, body, &[]).await
    }

    async fn send_with_query(
        &self,
        url: &str,
        ctx: &ClientContext,
        visitor: &str,
        mut body: Map<String, Value>,
        query: &[(&str, &str)],
    ) -> Result<Value> {
        body.insert("context".into(), ctx.to_json(&self.hl, &self.gl, visitor));
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static(ctx.user_agent));
        if url.starts_with(MUSIC_BASE) {
            headers.insert(ORIGIN, HeaderValue::from_static(MUSIC_ORIGIN));
            headers.insert("x-origin", HeaderValue::from_static(MUSIC_ORIGIN));
        }
        if let Some(referer) = ctx.referer {
            headers.insert(REFERER, HeaderValue::from_static(referer));
        }
        headers.insert(
            "X-Youtube-Bootstrap-Logged-In",
            HeaderValue::from_static("false"),
        );
        headers.insert("X-YouTube-Client-Name", HeaderValue::from(ctx.id));
        headers.insert(
            "X-YouTube-Client-Version",
            HeaderValue::from_static(ctx.version),
        );
        headers.insert("X-Goog-Api-Key", HeaderValue::from_static(API_KEY));
        if let Ok(v) = HeaderValue::from_str(visitor) {
            headers.insert("X-Goog-Visitor-Id", v);
        }
        headers.insert("X-Goog-Api-Format-Version", HeaderValue::from_static("2"));
        let response = self
            .http
            .post(url)
            .query(&[("key", API_KEY), ("prettyPrint", "false")])
            .query(query)
            .headers(headers)
            .json(&Value::Object(body))
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            tracing::debug!(
                "{url} -> {status}: {}",
                response.text().await.unwrap_or_default()
            );
            return Err(Error::Status(status.as_u16()));
        }
        Ok(response.json().await?)
    }
}
