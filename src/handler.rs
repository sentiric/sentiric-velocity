use crate::cache::CacheManager;
use crate::config;
use anyhow::Result;
use async_trait::async_trait;
use hyper::client::connect::dns::Name;
use hyper::{header, Body, HeaderMap, Method, Request, Response, StatusCode};
use lazy_static::lazy_static;
use reqwest::dns::Resolve;
use reqwest::Client;
use std::error::Error as StdError;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{info, warn};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

struct TrustDnsResolver(TokioAsyncResolver);

impl TrustDnsResolver {
    fn new() -> Self {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::cloudflare(), ResolverOpts::default())
                .unwrap();
        Self(resolver)
    }
}

#[async_trait]
impl Resolve for TrustDnsResolver {
    fn resolve(
        &self,
        name: Name,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn Iterator<Item = SocketAddr> + Send + 'static>, Box<dyn StdError + Send + Sync>>> + Send>>
    {
        let resolver = self.0.clone();
        Box::pin(async move {
            let addrs = resolver.lookup_ip(name.as_str()).await?;
            let mut sock_addrs: Vec<SocketAddr> =
                addrs.into_iter().map(|addr| (addr, 0).into()).collect();
            sock_addrs.dedup();
            let iter: Box<dyn Iterator<Item = SocketAddr> + Send + 'static> = Box::new(sock_addrs.into_iter());
            Ok(iter)
        })
    }
}


lazy_static! {
    static ref HTTP_CLIENT: Client = {
        let resolver = TrustDnsResolver::new();
        reqwest::Client::builder()
            .dns_resolver(Arc::new(resolver))
            .no_proxy()
            // İYİLEŞTİRME: Maksimum uyumluluk için giden isteklerde sadece HTTP/1.1 kullan.
            // Bu, Google gibi katı HTTP/2 sunucularındaki "protocol error" hatalarını çözer.
            .http1_only()
            .build()
            .expect("Failed to build reqwest client")
    };
}

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
            *builder.headers_mut().unwrap() = cached_entry.headers.clone();
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
    req: Request<Body>,
    cache: Arc<CacheManager>,
    cache_key: &str,
) -> Result<Response<Body>> {
    let config = config::get();
    let method = req.method().clone();
    let uri_string = req.uri().to_string();

    let mut headers = req.headers().clone();
    headers.remove(header::CONNECTION);
    headers.remove("keep-alive");
    headers.remove(header::PROXY_AUTHENTICATE);
    headers.remove(header::PROXY_AUTHORIZATION);
    headers.remove(header::TE);
    headers.remove(header::TRAILER);
    headers.remove(header::TRANSFER_ENCODING);
    headers.remove(header::UPGRADE);
    headers.remove("Proxy-Connection");

    let request_builder = HTTP_CLIENT
        .request(req.method().clone(), req.uri().to_string())
        .headers(headers)
        .body(hyper::body::to_bytes(req.into_body()).await?)
        .header(header::USER_AGENT, &config.proxy.user_agent);

    let response = request_builder.send().await?;

    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = response.bytes().await?;

    if status == StatusCode::OK && method == Method::GET {
        let mut headers_to_cache = HeaderMap::new();
        for (key, value) in headers.iter() {
            if key != header::CONNECTION && key != header::TRANSFER_ENCODING {
                headers_to_cache.insert(key.clone(), value.clone());
            }
        }

        cache
            .put(cache_key, &uri_string, body_bytes.to_vec(), headers_to_cache)
            .await;
    }

    let mut builder = Response::builder().status(status.as_u16());
    *builder.headers_mut().unwrap() = headers.clone();
    Ok(builder.body(Body::from(body_bytes))?)
}