use anyhow::Result;
use clap::{Parser, Subcommand}; // Düzeltildi
use std::sync::Arc;
use tracing::info;

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

    // Loglama altyapısını kur (config'den seviyeyi alarak)
    let log_level = settings.log.level.parse::<tracing_subscriber::filter::LevelFilter>()
        .unwrap_or(tracing_subscriber::filter::LevelFilter::INFO);
    tracing_subscriber::fmt().with_max_level(log_level).init();

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
            println!("Durdurmak için 'stop-proxy.bat' betiğini kullanın veya çalışan işlemi sonlandırın.");
        }
        Commands::Status => {
            println!("Proxy durum kontrolü henüz tam implemente edilmedi.");
            println!("Durumu görmek için yönetim arayüzünü ziyaret edin: http://127.0.0.1:8080");
        }
    }

    Ok(())
}