// src/handler.rs (Proxy Mantığı)

use hyper::{Method, Request, Body, Response, Client, Uri};
use anyhow::Result;
use tracing::{info, warn};
use tokio::io;

pub async fn proxy_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    info!("Gelen istek: {} {}", req.method(), req.uri());

    if Method::CONNECT == req.method() {
        // HTTPS Tünelleme
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
            // ... Hata response'u oluştur ...
            Ok(Response::new(Body::from("CONNECT Error")))
        }
    } else {
        // HTTP İstekleri
        // TODO: Cache kontrolü burada yapılacak
        // TODO: Whitelist kontrolü burada yapılacak
        let client = Client::new();
        match client.request(req).await {
            Ok(res) => Ok(res),
            Err(e) => {
                warn!("İstek yönlendirme hatası: {}", e);
                // ... Hata response'u oluştur ...
                Ok(Response::new(Body::from("Proxy Error")))
            }
        }
    }
}

// Helper fonksiyonlar (HTTPS tünelleme için)
fn host_addr(uri: &Uri) -> Option<String> {
    uri.authority().map(|auth| auth.to_string())
}

async fn tunnel(mut upgraded: hyper::upgrade::Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = tokio::net::TcpStream::connect(addr).await?;
    io::copy_bidirectional(&mut upgraded, &mut server).await?;
    Ok(())
}