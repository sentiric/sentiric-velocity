use crate::cache::{CacheManager, CacheStats};
use crate::config;
use serde::Serialize;
use std::sync::Arc;
use warp::Filter;
use tracing::info;

#[derive(Serialize)]
struct StatsResponse {
    status: String,
    stats: CacheStats,
}

pub async fn start_management_server(cache: Arc<CacheManager>) {
    let config = config::get();
    let addr = format!("{}:{}", config.management.bind_address, config.management.port)
        .parse::<std::net::SocketAddr>()
        .unwrap();

    // Warp filter'ı ile cache'i route'lara taşı
    let with_cache = warp::any().map(move || cache.clone());

    let index_html = warp::path::end().map(|| warp::reply::html(include_str!("../templates/index.html")));

    let stats_route = warp::path!("api" / "stats")
        .and(with_cache.clone())
        .and_then(get_stats_handler);

    let clear_route = warp::path!("api" / "clear")
        .and(warp::post())
        .and(with_cache.clone())
        .and_then(clear_cache_handler);

    let routes = index_html.or(stats_route).or(clear_route);
    
    warp::serve(routes).run(addr).await;
}

async fn get_stats_handler(cache: Arc<CacheManager>) -> Result<impl warp::Reply, warp::Rejection> {
    let stats = cache.get_stats().await;
    let response = StatsResponse {
        status: "running".to_string(),
        stats,
    };
    Ok(warp::reply::json(&response))
}

async fn clear_cache_handler(cache: Arc<CacheManager>) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Yönetim arayüzünden cache temizleme isteği alındı.");
    cache.clear().await;
    Ok(warp::reply::with_status("Cache başarıyla temizlendi.", warp::http::StatusCode::OK))
}