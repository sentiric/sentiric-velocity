use crate::cache::{CacheManager, CacheStats};
use crate::certs::CertificateAuthority;
use crate::config;
use futures_util::{SinkExt, StreamExt};
use http::{Response as HttpResponse, StatusCode}; // DÜZELTİLDİ
use serde::Serialize;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tracing::{info, Level, Subscriber, warn};
use tracing_subscriber::{layer::Context, Layer};
use warp::ws::{Message, WebSocket};
use warp::Filter;

// --- WebSocket Loglama ---
lazy_static::lazy_static! {
    static ref LOG_BROADCASTER: Sender<String> = broadcast::channel(100).0;
}

#[derive(Clone)]
pub struct BroadcastLayer;

impl<S> Layer<S> for BroadcastLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if *event.metadata().level() <= Level::INFO {
            let mut message = String::new();
            let mut visitor = StringVisitor { string: &mut message };
            event.record(&mut visitor);
            if !message.is_empty() {
                let _ = LOG_BROADCASTER.send(message);
            }
        }
    }
}

struct StringVisitor<'a> {
    string: &'a mut String,
}

impl<'a> tracing::field::Visit for StringVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.string.push_str(&format!("{:?}", value).trim_matches('"'));
        }
    }
}

// --- Warp Sunucusu ---

#[derive(Serialize)]
struct StatsResponse {
    status: String,
    stats: CacheStats,
}

#[derive(Debug)]
struct CustomRejection(String);
impl warp::reject::Reject for CustomRejection {}

pub async fn start_management_server(cache: Arc<CacheManager>, ca: Arc<CertificateAuthority>) {
    let config = config::get();
    let addr = format!("{}:{}", config.management.bind_address, config.management.port)
        .parse::<std::net::SocketAddr>()
        .unwrap();

    let with_cache = warp::any().map(move || cache.clone());
    let with_ca = warp::any().map(move || ca.clone());

    let index_html = warp::path::end().map(|| warp::reply::html(include_str!("../templates/index.html")));

    let api = warp::path("api");

    let stats_route = api.and(warp::path("stats"))
        .and(with_cache.clone())
        .and_then(get_stats_handler);

    let clear_route = api.and(warp::path("clear"))
        .and(warp::post())
        .and(with_cache.clone())
        .and_then(clear_cache_handler);

    let entries_route = api.and(warp::path("entries"))
        .and(warp::get())
        .and(with_cache.clone())
        .and_then(get_entries_handler);
    
    let delete_entry_route = api.and(warp::path!("entries" / String))
        .and(warp::delete())
        .and(with_cache.clone())
        .and_then(delete_entry_handler);

    let cert_route = api.and(warp::path("ca.crt"))
        .and(with_ca.clone())
        .and_then(download_cert_handler);

    let log_route = api.and(warp::path("logs")).and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(handle_websocket)
    });

    let routes = index_html
        .or(stats_route)
        .or(clear_route)
        .or(entries_route)
        .or(delete_entry_route)
        .or(cert_route)
        .or(log_route)
        .recover(handle_rejection);
    
    info!("✅ Yönetim arayüzü: http://{}", addr);
    warp::serve(routes).run(addr).await;
}

async fn get_stats_handler(cache: Arc<CacheManager>) -> Result<impl warp::Reply, warp::Rejection> {
    let stats = cache.get_stats().await;
    let response = StatsResponse { status: "Aktif".to_string(), stats };
    Ok(warp::reply::json(&response))
}

async fn clear_cache_handler(cache: Arc<CacheManager>) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Yönetim arayüzünden cache temizleme isteği alındı.");
    cache.clear().await;
    Ok(warp::reply::with_status("Cache başarıyla temizlendi.", StatusCode::OK))
}

async fn get_entries_handler(cache: Arc<CacheManager>) -> Result<impl warp::Reply, warp::Rejection> {
    match cache.get_all_entries().await {
        Ok(entries) => Ok(warp::reply::json(&entries)),
        Err(e) => {
            warn!("Cache girdileri alınamadı: {}", e);
            Err(warp::reject::custom(CustomRejection(e.to_string())))
        }
    }
}

async fn delete_entry_handler(key_hash: String, cache: Arc<CacheManager>) -> Result<impl warp::Reply, warp::Rejection> {
    match cache.delete_entry(&key_hash).await {
        Ok(_) => Ok(warp::reply::with_status("Girdi silindi.", StatusCode::OK)),
        Err(e) => {
            warn!("Cache girdisi silinemedi: {}", e);
            Err(warp::reject::custom(CustomRejection(e.to_string())))
        }
    }
}

async fn download_cert_handler(ca: Arc<CertificateAuthority>) -> Result<impl warp::Reply, warp::Rejection> {
    let cert_path = ca.get_ca_cert_path();
    match tokio::fs::read(cert_path).await {
        Ok(data) => {
            let response = HttpResponse::builder()
                .header("Content-Type", "application/x-x509-ca-cert")
                .header("Content-Disposition", "attachment; filename=\"VeloCache_CA.crt\"")
                .body(data)
                .unwrap();
            Ok(response)
        },
        Err(e) => {
            warn!("Sertifika dosyası okunamadı: {}", e);
            Err(warp::reject::not_found())
        }
    }
}

async fn handle_websocket(websocket: WebSocket) {
    let (mut tx, mut rx) = websocket.split();
    let mut log_rx = LOG_BROADCASTER.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = log_rx.recv().await {
            if tx.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });
    
    tokio::spawn(async move {
        while let Some(result) = rx.next().await {
            if result.is_err() { break; }
        }
    });
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else if let Some(e) = err.find::<CustomRejection>() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = e.0.clone();
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("UNHANDLED_REJECTION: {:?}", err);
    }
    
    let json = warp::reply::json(&serde_json::json!({
        "code": code.as_u16(),
        "message": message,
    }));

    Ok(warp::reply::with_status(json, code))
}