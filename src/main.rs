use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;
use std::sync::Arc;
use atty::Stream;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod cache;
mod certs;
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
    #[command(about = "Proxy sunucusunu ve yönetim arayüzünü başlatır")]
    Run,
}

#[tokio::main]
async fn main() -> Result<()> {
    config::init()?;
    let settings = config::get();

    let log_level = settings.log.level.parse::<tracing_subscriber::filter::LevelFilter>()
        .unwrap_or(tracing_subscriber::filter::LevelFilter::INFO);

    let use_ansi = env::var("NO_COLOR").is_err() && atty::is(Stream::Stdout);
    
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(use_ansi);

    let broadcast_layer = management::BroadcastLayer;

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(log_level)
        .with(broadcast_layer)
        .init();

    info!("VeloCache başlatılıyor...");

    let cert_authority = Arc::new(certs::CertificateAuthority::new()?);
    info!("Kök sertifika başarıyla yüklendi/oluşturuldu.");

    let cache_manager = Arc::new(cache::CacheManager::new(settings)?);

    let cli = Cli::parse();
    match &cli.command {
        Commands::Run => {
            let mgmt_cache = cache_manager.clone();
            let mgmt_ca = cert_authority.clone();
            tokio::spawn(management::start_management_server(mgmt_cache, mgmt_ca));
            
            proxy::start_server(cache_manager, cert_authority).await?;
        }
    }

    Ok(())
}