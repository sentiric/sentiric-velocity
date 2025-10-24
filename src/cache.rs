use crate::config::Settings;
use anyhow::{Context, Result};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::fs;
use std::num::NonZeroUsize;
use std::path::PathBuf; // Düzeltildi
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

#[derive(Serialize, Deserialize, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    created_at: SystemTime,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub in_memory_items: usize,
    pub disk_items: u64,
    pub total_disk_size_mb: u64,
}

pub struct CacheManager {
    memory_cache: Mutex<LruCache<String, CacheEntry>>,
    disk_path: Option<PathBuf>,
    ttl: Duration,
    stats: Arc<CacheStatsInternal>,
}

#[derive(Default)]
struct CacheStatsInternal {
    hits: AtomicU64,
    misses: AtomicU64,
    disk_items: AtomicU64,
    total_disk_size: AtomicU64,
}

impl CacheManager {
    pub fn new(config: &Settings) -> Result<Self> {
        let memory_items = NonZeroUsize::new(config.cache.memory_items)
            .context("Cache memory_items must be greater than 0")?;
        
        let disk_path = config.cache.disk_path.as_ref().map(PathBuf::from);
        if let Some(path) = &disk_path {
            fs::create_dir_all(path).context("Failed to create cache directory")?;
            info!("Disk cache enabled at: {:?}", path);
        }

        Ok(Self {
            memory_cache: Mutex::new(LruCache::new(memory_items)),
            disk_path,
            ttl: Duration::from_secs(config.cache.ttl_seconds),
            stats: Arc::new(CacheStatsInternal::default()),
        })
    }

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        entry.created_at.elapsed().unwrap_or_default() > self.ttl
    }

    fn key_to_path(&self, key: &str) -> Option<PathBuf> {
        self.disk_path.as_ref().map(|p| {
            // Basit bir hash'leme ile dosya adı oluşturuyoruz
            let hash = format!("{:x}", md5::compute(key));
            p.join(hash)
        })
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        // 1. Bellek cache'ini kontrol et
        let mut mem_cache = self.memory_cache.lock().await;
        if let Some(entry) = mem_cache.get(key) {
            if !self.is_expired(entry) {
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                debug!("CACHE HIT (memory): {}", key);
                return Some(entry.data.clone());
            } else {
                // Süresi dolmuş, bellekten sil
                mem_cache.pop(key);
            }
        }
        drop(mem_cache);

        // 2. Disk cache'ini kontrol et
        if let Some(path) = self.key_to_path(key) {
            if let Ok(file_content) = tokio::fs::read(&path).await {
                if let Ok(entry) = bincode::deserialize::<CacheEntry>(&file_content) {
                    if !self.is_expired(&entry) {
                        self.stats.hits.fetch_add(1, Ordering::Relaxed);
                        debug!("CACHE HIT (disk): {}", key);
                        // Diskte bulunanı belleğe de ekleyelim
                        let mut mem_cache = self.memory_cache.lock().await;
                        mem_cache.put(key.to_string(), entry.clone());
                        return Some(entry.data);
                    } else {
                        // Süresi dolmuş, diskten sil
                        let _ = tokio::fs::remove_file(path).await;
                    }
                }
            }
        }

        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        debug!("CACHE MISS: {}", key);
        None
    }

    pub async fn put(&self, key: &str, data: Vec<u8>) {
        let entry = CacheEntry {
            data,
            created_at: SystemTime::now(),
        };

        // 1. Diske yaz (eğer aktifse)
        if let Some(path) = self.key_to_path(key) {
             if let Ok(encoded) = bincode::serialize(&entry) {
                if tokio::fs::write(&path, &encoded).await.is_ok() {
                    self.stats.disk_items.fetch_add(1, Ordering::Relaxed);
                    self.stats.total_disk_size.fetch_add(encoded.len() as u64, Ordering::Relaxed);
                }
             } else {
                warn!("Failed to serialize cache entry for key: {}", key);
             }
        }
        
        // 2. Belleğe yaz
        let mut mem_cache = self.memory_cache.lock().await;
        mem_cache.put(key.to_string(), entry);
    }

    pub async fn clear(&self) {
        // Belleği temizle
        self.memory_cache.lock().await.clear();

        // Diski temizle
        if let Some(path) = &self.disk_path {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }

        // İstatistikleri sıfırla
        self.stats.hits.store(0, Ordering::Relaxed);
        self.stats.misses.store(0, Ordering::Relaxed);
        self.stats.disk_items.store(0, Ordering::Relaxed);
        self.stats.total_disk_size.store(0, Ordering::Relaxed);
        info!("Cache cleared successfully.");
    }

    pub async fn get_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.stats.hits.load(Ordering::Relaxed),
            misses: self.stats.misses.load(Ordering::Relaxed),
            in_memory_items: self.memory_cache.lock().await.len(),
            disk_items: self.stats.disk_items.load(Ordering::Relaxed),
            total_disk_size_mb: self.stats.total_disk_size.load(Ordering::Relaxed) / (1024 * 1024),
        }
    }
}