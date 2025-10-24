use serde::Deserialize;
use std::sync::OnceLock;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub proxy: Proxy,
    pub cache: Cache,
    pub management: Management,
    pub log: Log,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Proxy {
    pub port: u16,
    pub bind_address: String,
    pub user_agent: String,
    pub whitelist: Vec<String>,
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

static SETTINGS: OnceLock<Settings> = OnceLock::new();

pub fn init() -> Result<()> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?
        .try_deserialize()?;
    SETTINGS.set(settings).map_err(|_| anyhow::anyhow!("Configuration already initialized"))?;
    Ok(())
}

pub fn get() -> &'static Settings {
    SETTINGS.get().expect("Configuration is not initialized")
}