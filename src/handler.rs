use crate::cache::CacheManager;
use crate::config;
use anyhow::{anyhow, Result}; // anyhow import'u eklendi
use hyper::{Body, Client, Method, Request, Response, StatusCode, Uri, header};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::io;
use tracing::{info, warn}; // info import'u geri eklendi

pub async fn proxy_handler(
    req: Request<Body>,
    cache: Arc<CacheManager>,
) -> Result<Response<Body>, Infallible> {
    
    // Gelen her isteği loglayalım
    info!("Gelen istek: {} {}", req.method(), req.uri());
    
    // CONNECT metodunu (HTTPS Tünelleme) ele al
    if Method::CONNECT == req.method() {
        // handle_connect artık doğrudan bir Response<Body> döndürecek.
        // Hata durumlarını da bu fonksiyon içinde ele alıyoruz.
        return Ok(handle_connect(req).await);
    } 

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

// BU FONKSİYON TAMAMEN YENİLENDİ
async fn handle_connect(req: Request<Body>) -> Response<Body> {
    // 1. Hedef adresi al
    let addr = match host_addr(req.uri()) {
        Some(addr) => addr,
        None => {
            warn!("CONNECT isteği için hedef adresi anlaşılamadı: {}", req.uri());
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("CONNECT için host belirtilmemiş."))
                .unwrap();
        }
    };

    // 2. İsteği "upgrade" etmeye çalış ve I/O işlemini beklet
    tokio::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(upgraded) => {
                // Tünelleme işlemini yap
                if let Err(e) = tunnel(upgraded, addr).await {
                    warn!("HTTPS tünel hatası: {}", e);
                };
            }
            Err(e) => warn!("Upgrade hatası: {}", e),
        }
    });
    
    // 3. Tarayıcıya bağlantının başarılı olduğunu ve tünelin hazır olduğunu bildir.
    // Bu yanıtı hemen gönderiyoruz, tünelleme işlemi arka planda devam ediyor.
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

// ... dosyanın geri kalanı aynı ...
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

fn host_addr(uri: &Uri) -> Option<String> {
    uri.authority().map(|auth| auth.to_string())
}

async fn tunnel(mut upgraded: hyper::upgrade::Upgraded, addr: String) -> std::io::Result<()> {
    info!("Tünel başlatılıyor -> {}", addr);
    let mut server = tokio::net::TcpStream::connect(addr).await?;
    let (from_client, from_server) = io::copy_bidirectional(&mut upgraded, &mut server).await?;
    info!("Tünel kapatıldı. İstemciden {} bayt, sunucudan {} bayt aktarıldı.", from_client, from_server);
    Ok(())
}