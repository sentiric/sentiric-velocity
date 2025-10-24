use crate::{cache::CacheManager, config, handler::proxy_handler, management};
use anyhow::Result;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

pub async fn start_server(cache_manager: Arc<CacheManager>) -> Result<()> {
    let config = config::get();

    let management_cache = cache_manager.clone();
    tokio::spawn(management::start_management_server(management_cache));

    let addr = format!("{}:{}", config.proxy.bind_address, config.proxy.port).parse::<SocketAddr>()?;

    let listener = TcpListener::bind(addr).await?;
    info!("🚀 VeloCache proxy sunucusu başlatıldı: http://{}", addr);
    info!("✅ Yönetim arayüzü: http://{}:{}", config.management.bind_address, config.management.port);

    loop {
        let (stream, client_addr) = listener.accept().await?;
        let cache = cache_manager.clone();
        
        tokio::spawn(async move {
            if let Err(e) = serve_connection(stream, cache).await {
                warn!("Bağlantı hatası from {}: {}", client_addr, e);
            }
        });
    }
}

async fn serve_connection(mut stream: TcpStream, cache: Arc<CacheManager>) -> Result<()> {
    // Gelen ilk veriyi oku ve CONNECT metodu olup olmadığını kontrol et
    let mut buffer = [0; 4096];
    let n = stream.peek(&mut buffer).await?;

    // Eğer istek bir CONNECT metodu ise, manuel tünelleme yap
    if buffer[..n].starts_with(b"CONNECT") {
        handle_connect_manually(stream, &buffer[..n]).await?;
    } else {
        // Değilse, Hyper'ın yönetimine devret
        let service = service_fn(move |req| proxy_handler(req, cache.clone()));
        Http::new()
            .serve_connection(stream, service)
            .with_upgrades() // with_upgrades() eklemek önemlidir
            .await?;
    }
    Ok(())
}

async fn handle_connect_manually(mut stream: TcpStream, request_bytes: &[u8]) -> Result<()> {
    // Request'ten host adresini ve portunu çıkar
    let req_str = String::from_utf8_lossy(request_bytes);
    let host = req_str
        .lines()
        .find(|line| line.to_lowercase().starts_with("connect "))
        .and_then(|line| line.split_whitespace().nth(1));

    if let Some(host) = host {
        info!("Manuel tünel isteği -> {}", host);
        // İstemciye tünelin kurulduğunu bildir
        stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
        
        // Hedef sunucuya bağlan
        match TcpStream::connect(host).await {
            Ok(mut server_stream) => {
                // İstemci ve sunucu arasında veri kopyalamaya başla
                let (from_client, from_server) =
                    io::copy_bidirectional(&mut stream, &mut server_stream).await?;
                info!(
                    "Tünel kapatıldı: {}. İstemciden: {} bayt, Sunucudan: {} bayt.",
                    host, from_client, from_server
                );
            }
            Err(e) => {
                error!("Hedefe bağlanılamadı ({}): {}", host, e);
                // İstemciye hata mesajı gönder
                stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await?;
            }
        }
    } else {
        warn!("Geçersiz CONNECT isteği alındı.");
        stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await?;
    }
    Ok(())
}