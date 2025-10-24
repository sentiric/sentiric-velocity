use clap::{Parser, Subcommand};
use anyhow::Result;

mod config;
mod cache;
mod proxy;
mod handler;
mod management;

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
    let cli = Cli::parse();

    // Loglama altyapısını kur
    tracing_subscriber::fmt::init();

    match &cli.command {
        Commands::Run => {
            proxy::start_server().await?;
        }
        Commands::Stop => {
            // TODO: Graceful shutdown implementasyonu
            println!("Proxy durdurma komutu henüz tam implemente edilmedi.");
        }
        Commands::Status => {
            // TODO: Yönetim API'sine bağlanıp durum alma
             println!("Proxy durum kontrolü henüz tam implemente edilmedi.");
        }
    }

    Ok(())
}