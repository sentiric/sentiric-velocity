use crate::{cache::CacheManager, config, management};
use anyhow::Result;
use hyper::server::conn::Http;
use hyper::service::service_fn; // Düzeltildi
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

// Fonksiyon artık cache_manager'ı argüman olarak alıyor
pub async fn start_server(cache_manager: Arc<CacheManager>) -> Result<()> {
    let config = config::get();

    // Yönetim sunucusunu ayrı bir görevde başlat ve cache'i ona da ver
    let management_cache = cache_manager.clone();
    tokio::spawn(management::start_management_server(management_cache));

    let addr = format!("{}:{}", config.proxy.bind_address, config.proxy.port).parse::<SocketAddr>()?;

    let listener = TcpListener::bind(addr).await?;
    info!("🚀 VeloCache proxy sunucusu başlatıldı: http://{}", addr);
    info!("✅ Yönetim arayüzü: http://{}:{}", config.management.bind_address, config.management.port);

    loop {
        let (stream, _) = listener.accept().await?;
        
        // Her yeni bağlantı için cache'in bir kopyasını (Arc clone) al
        let cache_clone = cache_manager.clone();
        
        tokio::spawn(async move {
            // service_fn'e cache'i de geçir
            let service = service_fn(move |req| {
                crate::handler::proxy_handler(req, cache_clone.clone())
            });

            if let Err(err) = Http::new().serve_connection(stream, service).await {
                // Bağlantı kapandığında gelen "error shutting down" hatasını görmezden gel
                if !err.to_string().contains("error shutting down") {
                    error!("İstemci bağlantısında hata: {}", err);
                }
            }
        });
    }
}