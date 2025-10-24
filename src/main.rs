use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tracing::info;
use atty::Stream;
use std::env;

mod cache;
mod config;
mod handler;
mod management;
mod proxy;

#[derive(Parser)]
#[command(name = "VeloCache")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Proxy sunucusunu ve yönetim arayüzünü başlatır
    Run,
    /// Proxy'yi durdurur
    Stop,
    /// Proxy'nin durumunu kontrol eder
    Status,
}


#[tokio::main]
async fn main() -> Result<()> {
    // Config'i en başta yükle
    config::init()?;
    let settings = config::get();

    // Loglama altyapısını kur
    let log_level = settings.log.level.parse::<tracing_subscriber::filter::LevelFilter>()
        .unwrap_or(tracing_subscriber::filter::LevelFilter::INFO);

    // NO_COLOR ortam değişkeni varsa veya çıktı bir terminal değilse ANSI'yi devre dışı bırak.
    let use_ansi = env::var("NO_COLOR").is_err() && atty::is(Stream::Stdout);

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_ansi(use_ansi)
        .init();

    info!("VeloCache başlatılıyor...");

    // Paylaşılacak cache yöneticisini oluştur
    let cache_manager = Arc::new(cache::CacheManager::new(settings)?);

    let cli = Cli::parse();
    match &cli.command {
        Commands::Run => {
            proxy::start_server(cache_manager).await?;
        }
        Commands::Stop => {
            println!("Proxy durdurma komutu henüz tam implemente edilmedi.");
            println!("Durdurmak için 'stop.bat' (Windows) veya 'stop.sh' (Linux) betiğini kullanın.");
        }
        Commands::Status => {
            println!("Proxy durum kontrolü henüz tam implemente edilmedi.");
            println!("Durumu görmek için yönetim arayüzünü ziyaret edin: http://127.0.0.1:8080");
        }
    }

    Ok(())
}