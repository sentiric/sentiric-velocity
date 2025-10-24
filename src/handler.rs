use crate::cache::CacheManager;
use crate::config;
use anyhow::Result;
use hyper::{header, Body, Client, Method, Request, Response, StatusCode};
use hyper_tls::HttpsConnector;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn proxy_handler(
    req: Request<Body>,
    cache: Arc<CacheManager>,
) -> Result<Response<Body>, hyper::Error> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let cache_key = format!("{}::{}", method, uri);

    info!("İstek: {} {}", method, uri);

    if method == Method::GET {
        if let Some(cached_entry) = cache.get(&cache_key).await {
            let mut builder = Response::builder().status(StatusCode::OK);
            if let Ok(val) = header::HeaderValue::from_str(&cached_entry.content_type) {
                builder = builder.header(header::CONTENT_TYPE, val);
            }
            return Ok(builder.body(Body::from(cached_entry.data)).unwrap());
        }
    }

    match forward_request(req, cache, &cache_key).await {
        Ok(res) => Ok(res),
        Err(e) => {
            warn!("İstek yönlendirme hatası: {}", e);
            let resp = Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!("Proxy hatası: {}", e)))
                .unwrap();
            Ok(resp)
        }
    }
}

async fn forward_request(
    mut req: Request<Body>,
    cache: Arc<CacheManager>,
    cache_key: &str,
) -> Result<Response<Body>> {
    let config = config::get();
    let method = req.method().clone();
    let uri_string = req.uri().to_string();

    req.headers_mut().insert(
        header::USER_AGENT,
        header::HeaderValue::from_str(&config.proxy.user_agent)?,
    );

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let response = client.request(req).await?;

    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = hyper::body::to_bytes(response.into_body()).await?;

    if status == StatusCode::OK && method == Method::GET {
        let content_type = headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        cache
            .put(cache_key, &uri_string, body_bytes.to_vec(), &content_type)
            .await;
    }

    let mut builder = Response::builder().status(status);
    *builder.headers_mut().unwrap() = headers;
    Ok(builder.body(Body::from(body_bytes))?)
}