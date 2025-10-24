// src/proxy.rs

use crate::{config, management};
use anyhow::Result;
use hyper::server::conn::Http;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

pub async fn start_server() -> Result<()> {
    // Önce config'i yükle
    config::init()?;
    let config = config::get();

    // Yönetim sunucusunu ayrı bir görevde başlat
    tokio::spawn(management::start_management_server());

    let addr = format!("{}:{}", config.proxy.bind_address, config.proxy.port).parse::<SocketAddr>()?;

    let listener = TcpListener::bind(addr).await?;
    info!("🚀 VeloCache proxy sunucusu başlatıldı: http://{}", addr);
    info!("✅ Yönetim arayüzü: http://{}:{}", config.management.bind_address, config.management.port);

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let service = service_fn(move |req| crate::handler::proxy_handler(req));
            if let Err(err) = Http::new().serve_connection(stream, service).await {
                error!("İstemci bağlantısında hata: {}", err);
            }
        });
    }
}


