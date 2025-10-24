use crate::{cache::CacheManager, certs::CertificateAuthority, handler::proxy_handler};
use anyhow::{Context, Result};
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Uri};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener; // UYARI DÜZELTME: Kullanılmayan TcpStream kaldırıldı.
use tokio_rustls::TlsAcceptor;
use tracing::{info, warn, Instrument};

pub async fn start_server(cache: Arc<CacheManager>, ca: Arc<CertificateAuthority>) -> Result<()> {
    let config = crate::config::get();
    let addr: SocketAddr = format!("{}:{}", config.proxy.bind_address, config.proxy.port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("🚀 VeloCache proxy sunucusu başlatıldı: http://{}", addr);

    loop {
        let (stream, client_addr) = listener.accept().await?;
        let cache = cache.clone();
        let ca = ca.clone();

        tokio::spawn(
            async move {
                let service = service_fn(move |req| {
                    serve_req(req, cache.clone(), ca.clone())
                });

                if let Err(e) = Http::new()
                    .http1_only(true)
                    .http1_keep_alive(true)
                    .serve_connection(stream, service)
                    .with_upgrades()
                    .await
                {
                    if !e.to_string().contains("body write") 
                        && !e.to_string().contains("aborted")
                        && !e.to_string().contains("end of file")
                        && !e.to_string().contains("connection reset")
                    {
                         warn!("Bağlantı hatası: {}", e);
                    }
                }
            }
            .instrument(tracing::info_span!("client", %client_addr)),
        );
    }
}

async fn serve_req(
    req: Request<Body>,
    cache: Arc<CacheManager>,
    ca: Arc<CertificateAuthority>,
) -> Result<Response<Body>, hyper::Error> {
    if Method::CONNECT == req.method() {
        if let Some(host) = req.uri().authority().map(|auth| auth.to_string()) {
            tokio::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = handle_connect(upgraded, host, cache, ca).await {
                             warn!("HTTPS tünel hatası: {}", e);
                        }
                    }
                    Err(e) => warn!("Upgrade hatası: {}", e),
                }
            });
            Ok(Response::new(Body::empty()))
        } else {
            warn!("CONNECT isteğinde host bulunamadı: {:?}", req.uri());
            let mut resp = Response::new(Body::from("CONNECT must be to a socket address"));
            *resp.status_mut() = http::StatusCode::BAD_REQUEST;
            Ok(resp)
        }
    } else {
        proxy_handler(req, cache).await
    }
}

async fn handle_connect(
    upgraded: hyper::upgrade::Upgraded,
    host: String,
    cache: Arc<CacheManager>,
    ca: Arc<CertificateAuthority>,
) -> Result<()> {
    info!("HTTPS Intercept -> {}", host);
    
    let acceptor = TlsAcceptor::from(ca.get_server_config(&host.split(':').next().unwrap_or(&host))?);
    let stream = acceptor.accept(upgraded).await.context("TLS Handshake hatası")?;

    let service = service_fn(move |mut req: Request<Body>| {
        let host = host.clone();
        let cache = cache.clone();
        async move {
            let authority = host.parse::<http::uri::Authority>().unwrap();
            let uri = Uri::builder()
                .scheme("https")
                .authority(authority)
                .path_and_query(req.uri().path_and_query().unwrap().clone())
                .build()
                .unwrap();
            *req.uri_mut() = uri;
            proxy_handler(req, cache).await
        }
    });

    if let Err(e) = Http::new()
        .http1_only(true)
        .http1_keep_alive(true)
        .serve_connection(stream, service)
        .await {
            if !e.to_string().contains("body write") && !e.to_string().contains("aborted") {
                warn!("Intercepted HTTPS bağlantı hatası: {}", e);
            }
        }
        
    Ok(())
}