use serde::Deserialize;
use std::env;
use std::sync::OnceLock;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub proxy: Proxy,
    pub cache: Cache,
    pub management: Management,
    pub log: Log,
    pub certs: Certs,
    // YENİ: 'rules' bölümünü ekliyoruz. 'Option' kullanarak isteğe bağlı hale getiriyoruz.
    #[serde(default)] // config.toml'da [rules] yoksa hata vermemesi için
    pub rules: Rules,
}

#[derive(Debug, Deserialize, Clone, Default)] // 'Default' ekledik
pub struct Rules {
    // YENİ: 'ignore_hosts' listesi. 'Option' kullanarak isteğe bağlı hale getiriyoruz.
    #[serde(default)] // [rules] altında ignore_hosts yoksa hata vermemesi için
    pub ignore_hosts: Vec<String>,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Proxy {
    pub port: u16,
    pub bind_address: String,
    pub user_agent: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Cache {
    pub memory_items: usize,
    pub disk_path: Option<String>,
    pub ttl_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Management {
    pub port: u16,
    pub bind_address: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Certs {
    pub path: String,
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

pub fn init() -> Result<()> {
    // NİHAİ DÜZELTME: config.toml dosyasını esnek bir şekilde ara.
    // 1. Mevcut çalışma dizininde ara (cargo run, manual exe).
    // 2. .exe'nin yanında ara (portable dağıtım).
    // 3. .exe'nin bir üst dizininde ara (start.bat kullanımı).
    let config_path = if std::path::Path::new("config.toml").exists() {
        "config.toml".into()
    } else {
        let exe_path = env::current_exe().context("Failed to get current executable path")?;
        let exe_dir = exe_path.parent().context("Failed to get executable directory")?;
        
        if exe_dir.join("config.toml").exists() {
            exe_dir.join("config.toml")
        } else {
            let parent_dir = exe_dir.parent().context("Failed to get parent directory")?;
            parent_dir.join("config.toml")
        }
    };
    
    let settings = config::Config::builder()
        .add_source(config::File::from(config_path.clone()).required(true))
        .build()
        .with_context(|| format!("Failed to build configuration from {:?}", config_path))?
        .try_deserialize()
        .context("Failed to deserialize configuration")?;

    SETTINGS.set(settings).map_err(|_| anyhow::anyhow!("Configuration already initialized"))?;
    Ok(())
}


pub fn get() -> &'static Settings {
    SETTINGS.get().expect("Configuration is not initialized")
}