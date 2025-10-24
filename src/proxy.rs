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
                if !e.to_string().contains("broken pipe") 
                   && !e.to_string().contains("os error 10053") 
                   && !e.to_string().contains("os error 10054") {
                    warn!("Bağlantı hatası from {}: {}", client_addr, e);
                }
            }
        });
    }
}

async fn serve_connection(mut stream: TcpStream, cache: Arc<CacheManager>) -> Result<()> {
    let mut buffer = [0; 4096];
    let n = stream.peek(&mut buffer).await?;

    if buffer[..n].starts_with(b"CONNECT") {
        handle_connect_manually(stream).await?;
    } else {
        // HTTP istekleri için stream'i doğrudan Hyper'a ver.
        let service = service_fn(move |req| proxy_handler(req, cache.clone()));
        Http::new()
            .serve_connection(stream, service)
            .with_upgrades() // WebSocket gibi protokoller için upgrade'i etkin bırak
            .await?;
    }
    Ok(())
}

async fn handle_connect_manually(mut stream: TcpStream) -> Result<()> {
    // Önce istemciden gelen tüm HTTP başlığını okuyup atlayalım.
    // Bu, stream'i temizler ve sadece tünellenecek veriyi bırakır.
    let mut buffer = vec![0; 4096];
    let mut pos = 0;
    
    // Geçici bir buffer'a başlığı oku
    let n_read = loop {
        let n = stream.read(&mut buffer[pos..]).await?;
        if n == 0 { 
            // Bağlantı başlık bitmeden kapandı
            warn!("CONNECT isteği sırasında bağlantı erken kapandı.");
            return Ok(());
        }
        pos += n;
        // HTTP başlığının sonu "\r\n\r\n" dir.
        if buffer[..pos].windows(4).any(|window| window == b"\r\n\r\n") {
            break pos;
        }
        if pos >= buffer.len() {
            // Başlık çok büyük, buffer'ı büyüt (nadiren olur)
            buffer.resize(buffer.len() * 2, 0);
        }
    };
    
    let request_str = String::from_utf8_lossy(&buffer[..n_read]);
    let host = request_str
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
                let (from_client, from_server) =
                    io::copy_bidirectional(&mut stream, &mut server_stream).await?;
                info!(
                    "Tünel kapatıldı: {}. İstemciden: {} bayt, Sunucudan: {} bayt.",
                    host, from_client, from_server
                );
            }
            Err(e) => {
                error!("Hedefe bağlanılamadı ({}): {}", host, e);
            }
        }
    } else {
        warn!("Geçersiz CONNECT isteği alındı (Hedef bulunamadı).");
    }
    Ok(())
}