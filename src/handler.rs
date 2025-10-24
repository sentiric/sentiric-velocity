use crate::cache::CacheManager;
use crate::config;
use anyhow::Result;
use hyper::{Body, Client, Method, Request, Response, StatusCode, Uri};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::io;
use tracing::warn; // Düzeltildi

// İmza güncellendi: cache parametresi eklendi
pub async fn proxy_handler(
    req: Request<Body>,
    cache: Arc<CacheManager>,
) -> Result<Response<Body>, Infallible> {
    // Whitelist kontrolü
    let host = req.uri().host().unwrap_or_default();
    let config = config::get();
    if !config.proxy.whitelist.is_empty() && !config.proxy.whitelist.iter().any(|domain| host.ends_with(domain)) {
        warn!("Bloklanan istek (whitelist dışı): {}", host);
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Bu alan adı whitelist'te değil."))
            .unwrap());
    }

    if Method::CONNECT == req.method() {
        // HTTPS Tünelleme (Bu kısım aynı kalabilir, cache'lenmez)
        match handle_connect(req).await {
            Ok(res) => Ok(res),
            Err(e) => {
                warn!("CONNECT hatası: {}", e);
                Ok(Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(Body::from(format!("CONNECT hatası: {}", e)))
                    .unwrap())
            }
        }
    } else {
        // HTTP İstekleri (Cache mantığı burada)
        let cache_key = req.uri().to_string();

        // 1. Cache'i kontrol et
        if let Some(cached_data) = cache.get(&cache_key).await {
            return Ok(Response::new(Body::from(cached_data)));
        }

        // 2. Cache'de yoksa, isteği yönlendir
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
}

async fn forward_http_request(
    req: Request<Body>,
    cache: &Arc<CacheManager>,
    cache_key: &str,
) -> Result<Response<Body>> {
    let client = Client::new();
    let response = client.request(req).await?;

    if response.status() == StatusCode::OK {
        let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
        
        // Başarılı cevabı cache'e kaydet
        cache.put(cache_key, body_bytes.to_vec()).await;

        Ok(Response::new(Body::from(body_bytes)))
    } else {
        Ok(response)
    }
}


async fn handle_connect(req: Request<Body>) -> Result<Response<Body>> {
    if let Some(addr) = host_addr(req.uri()) {
        tokio::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, addr).await {
                        warn!("HTTPS tünel hatası: {}", e);
                    };
                }
                Err(e) => warn!("Upgrade hatası: {}", e),
            }
        });
        Ok(Response::new(Body::empty()))
    } else {
        warn!("CONNECT isteği için hedef adresi anlaşılamadı: {}", req.uri());
        Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("CONNECT için host belirtilmemiş."))
            .unwrap())
    }
}


// Helper fonksiyonlar (aynı kalır)
fn host_addr(uri: &Uri) -> Option<String> {
    uri.authority().map(|auth| auth.to_string())
}
async fn tunnel(mut upgraded: hyper::upgrade::Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = tokio::net::TcpStream::connect(addr).await?;
    io::copy_bidirectional(&mut upgraded, &mut server).await?;
    Ok(())
}