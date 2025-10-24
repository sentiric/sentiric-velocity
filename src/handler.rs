use crate::cache::CacheManager;
use crate::config;
use anyhow::Result;
use futures_util::stream::StreamExt;
use hyper::client::connect::dns::Name;
use hyper::client::HttpConnector;
use hyper::service::Service;
use hyper::{header, Body, Client, HeaderMap, Method, Request, Response, StatusCode};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use lazy_static::lazy_static;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tracing::{info, warn};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

// hyper::service::Service<Name> trait'ini uygulayan adaptör.
#[derive(Clone)]
struct TrustDnsResolver(TokioAsyncResolver);

impl Service<Name> for TrustDnsResolver {
    type Response = std::vec::IntoIter<SocketAddr>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, name: Name) -> Self::Future {
        let resolver = self.0.clone();
        Box::pin(async move {
            let addrs = resolver.lookup_ip(name.as_str()).await?;
            let sock_addrs: Vec<SocketAddr> =
                addrs.into_iter().map(|addr| (addr, 0).into()).collect();
            Ok(sock_addrs.into_iter())
        })
    }
}


lazy_static! {
    static ref HTTP_CLIENT: Client<HttpsConnector<HttpConnector<TrustDnsResolver>>> = {
        let resolver = TrustDnsResolver(
            TokioAsyncResolver::tokio(
                ResolverConfig::cloudflare(),
                ResolverOpts::default(),
            ).unwrap()
        );

        let mut http = HttpConnector::new_with_resolver(resolver);
        http.enforce_http(false);

        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .wrap_connector(http);

        Client::builder().build(https)
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
    mut req: Request<Body>,
    cache: Arc<CacheManager>,
    cache_key: &str,
) -> Result<Response<Body>> {
    let config = config::get();
    let method = req.method().clone();
    let uri_string = req.uri().to_string();

    req.headers_mut().remove(header::CONNECTION);
    req.headers_mut().remove("keep-alive");
    req.headers_mut().remove(header::PROXY_AUTHENTICATE);
    req.headers_mut().remove(header::PROXY_AUTHORIZATION);
    req.headers_mut().remove(header::TE);
    req.headers_mut().remove(header::TRAILER);
    req.headers_mut().remove(header::TRANSFER_ENCODING);
    req.headers_mut().remove(header::UPGRADE);
    req.headers_mut().remove("Proxy-Connection");

    req.headers_mut()
        .insert(header::USER_AGENT, config.proxy.user_agent.parse()?);

    let response = HTTP_CLIENT.request(req).await?;

    let status = response.status();
    let headers = response.headers().clone();

    let (mut sender, client_body) = Body::channel();

    let should_cache = status == StatusCode::OK && method == Method::GET;
    let mut body_stream = response.into_body();

    let cache_key_owned = cache_key.to_string();
    let headers_clone_for_cache = headers.clone();

    tokio::spawn(async move {
        let mut body_buffer = if should_cache { Some(Vec::new()) } else { None };

        while let Some(chunk_result) = body_stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    if let Some(buffer) = body_buffer.as_mut() {
                        buffer.extend_from_slice(&bytes);
                    }
                    if sender.send_data(bytes).await.is_err() {
                        warn!("İstemci bağlantısı kapandı, stream sonlandırılıyor.");
                        break;
                    }
                }
                Err(e) => {
                    warn!("Upstream'den veri okunurken hata: {}", e);
                    break;
                }
            }
        }

        if let Some(buffer) = body_buffer {
            let mut headers_to_cache = HeaderMap::new();
            for (key, value) in headers_clone_for_cache.iter() {
                if key != header::CONNECTION && key != header::TRANSFER_ENCODING {
                    headers_to_cache.insert(key.clone(), value.clone());
                }
            }
            cache
                .put(&cache_key_owned, &uri_string, buffer, headers_to_cache)
                .await;
        }
    });

    let mut builder = Response::builder().status(status);
    *builder.headers_mut().unwrap() = headers;
    Ok(builder.body(client_body)?)
}