// src/management.rs

use crate::config;
use std::convert::Infallible;
use warp::Filter;
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
struct StatsResponse {
    status: String,
    // TODO: Bu alanlar gerçek cache verileriyle doldurulacak
    cache_size_mb: usize,
    item_count: usize,
    hits: u64,
    misses: u64,
}

pub async fn start_management_server() {
    let config = config::get();
    let addr = format!("{}:{}", config.management.bind_address, config.management.port)
        .parse::<std::net::SocketAddr>()
        .unwrap();

    let index_html = warp::path::end().map(|| warp::reply::html(include_str!("../templates/index.html")));

    let stats_route = warp::path!("api" / "stats").map(|| {
        // TODO: Gerçek cache istatistiklerini buradan döndür
        let stats = StatsResponse {
            status: "running".to_string(),
            cache_size_mb: 123,
            item_count: 456,
            hits: 100,
            misses: 10,
        };
        warp::reply::json(&stats)
    });

    let clear_route = warp::path!("api" / "clear").and(warp::post()).map(|| {
        // TODO: Cache temizleme fonksiyonunu burada çağır
        info!("Yönetim arayüzünden cache temizleme isteği alındı.");
        "Cache temizlendi"
    });

    let routes = index_html.or(stats_route).or(clear_route);

    warp::serve(routes).run(addr).await;
}