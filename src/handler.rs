use crate::cache::CacheManager;
use crate::config;
use anyhow::Result;
use hyper::{Body, Client, Request, Response, StatusCode, header};
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{info, warn};

// FONKSİYON BASİTLEŞTİRİLDİ
pub async fn proxy_handler(
    req: Request<Body>,
    cache: Arc<CacheManager>,
) -> Result<Response<Body>, Infallible> {
    
    info!("Gelen istek (HTTP): {} {}", req.method(), req.uri());
    
    // HTTP isteklerini ele al
    let cache_key = req.uri().to_string();

    if let Some(cached_data) = cache.get(&cache_key).await {
        return Ok(Response::new(Body::from(cached_data)));
    }

    match forward_http_request(req, &cache, &cache_key).await {
        Ok(res) => Ok(res),
        Err(e) => {
            warn!("İstek yönlendirme hatası: {}", e);
            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!("Proxy hatası: {}", e)))
                .unwrap())
        }
    }
}

// BU FONKSİYON DEĞİŞMEDİ
async fn forward_http_request(
    mut req: Request<Body>,
    cache: &Arc<CacheManager>,
    cache_key: &str,
) -> Result<Response<Body>> {
    let config = config::get();
    
    req.headers_mut().remove(header::HOST);
    req.headers_mut().insert(
        header::USER_AGENT, 
        header::HeaderValue::from_str(&config.proxy.user_agent)?
    );

    let client = Client::new();
    let response = client.request(req).await?;

    if response.status() == StatusCode::OK {
        let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
        cache.put(cache_key, body_bytes.to_vec()).await;
        Ok(Response::new(Body::from(body_bytes)))
    } else {
        Ok(response)
    }
}