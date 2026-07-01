pub mod models;

use std::time::{Duration, Instant};

use reqwest::Client;

use models::{KeyValue, Method, RequestModel};

#[allow(dead_code)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<KeyValue>,
    pub body: String,
    pub duration: Duration,
}

pub async fn send_request(req: &RequestModel) -> Result<Response, String> {
    let client = Client::new();
    let url = build_url(req);

    let method = match req.method {
        Method::Get => reqwest::Method::GET,
        Method::Post => reqwest::Method::POST,
        Method::Put => reqwest::Method::PUT,
        Method::Delete => reqwest::Method::DELETE,
        Method::Patch => reqwest::Method::PATCH,
    };

    let mut builder = client.request(method, &url);

    for kv in &req.headers {
        if !kv.key.is_empty() {
            builder = builder.header(&kv.key, &kv.value);
        }
    }

    if !req.body.is_empty() {
        builder = builder.body(req.body.clone());
    }

    let start = Instant::now();
    let resp = builder.send().await.map_err(|e| e.to_string())?;
    let duration = start.elapsed();

    let status = resp.status().as_u16();
    let status_text = resp.status().canonical_reason().unwrap_or("").to_string();
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| KeyValue {
            key: k.to_string(),
            value: v.to_str().unwrap_or("").to_string(),
        })
        .collect();
    let body = resp.text().await.map_err(|e| e.to_string())?;

    Ok(Response {
        status,
        status_text,
        headers,
        body,
        duration,
    })
}

fn build_url(req: &RequestModel) -> String {
    let params: Vec<_> = req
        .params
        .iter()
        .filter(|kv| !kv.key.is_empty())
        .map(|kv| format!("{}={}", kv.key, kv.value))
        .collect();

    if params.is_empty() {
        req.url.clone()
    } else {
        let sep = if req.url.contains('?') { "&" } else { "?" };
        format!("{}{}{}", req.url, sep, params.join("&"))
    }
}
