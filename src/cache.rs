use crate::config::Settings;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use http_serde;
use hyper::HeaderMap;
use lru::LruCache;
use serde::{Deserialize, Serialize};

use std::fs;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

// Veri ve meta veriyi ayırmak için diskteki format
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheEntryDisk {
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,
    pub created_at: DateTime<Utc>,
    pub url: String,
}

// Bellekte tutulacak tam yapı
#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub headers: HeaderMap,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub data: Vec<u8>,
}

// UI için sadece meta verileri içeren yeni struct
#[derive(Serialize, Clone, Debug)]
pub struct CacheEntryMetadata {
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub size: u64,
    // YENİ: UI'da silme işlemi için hash'i de gönderiyoruz.
    pub hash: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub total_requests: u64,
    pub in_memory_items: usize,
    pub disk_items: u64,
    pub total_disk_size_bytes: u64,
    pub data_served_from_cache_bytes: u64,
}

pub struct CacheManager {
    memory_cache: Mutex<LruCache<String, Arc<CacheEntry>>>,
    disk_path: Option<PathBuf>,
    ttl: Duration,
    stats: Arc<CacheStatsInternal>,
}

#[derive(Default)]
struct CacheStatsInternal {
    hits: AtomicU64,
    misses: AtomicU64,
    disk_items: AtomicU64,
    total_disk_size_bytes: AtomicU64,
    data_served_from_cache_bytes: AtomicU64,
}

impl CacheManager {
    pub fn new(config: &Settings) -> Result<Self> {
        let memory_items = NonZeroUsize::new(config.cache.memory_items)
            .context("Cache memory_items must be greater than 0")?;
        
        let disk_path = config.cache.disk_path.as_ref().and_then(|p| {
            if p.is_empty() { None } else { Some(PathBuf::from(p)) }
        });

        if let Some(path) = &disk_path {
            fs::create_dir_all(path).context("Failed to create cache directory")?;
            info!("Disk cache enabled at: {:?}", path);
        }

        let cache = Self {
            memory_cache: Mutex::new(LruCache::new(memory_items)),
            disk_path,
            ttl: Duration::from_secs(config.cache.ttl_seconds),
            stats: Arc::new(CacheStatsInternal::default()),
        };

        if let Some(path) = &cache.disk_path {
             if let Ok(entries) = fs::read_dir(path) {
                let mut count = 0;
                let mut total_size = 0;
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            count += 1;
                            total_size += metadata.len();
                        }
                    }
                }
                cache.stats.disk_items.store(count, Ordering::Relaxed);
                cache.stats.total_disk_size_bytes.store(total_size, Ordering::Relaxed);
            }
        }
        
        Ok(cache)
    }

    fn is_expired(&self, created_at: &DateTime<Utc>) -> bool {
        (Utc::now() - *created_at).to_std().unwrap_or_default() > self.ttl
    }

    fn key_to_hash(&self, key: &str) -> String {
        format!("{:x}", md5::compute(key))
    }

    fn key_to_path(&self, key: &str) -> Option<PathBuf> {
        self.disk_path.as_ref().map(|p| p.join(self.key_to_hash(key)))
    }

    pub async fn get(&self, key: &str) -> Option<Arc<CacheEntry>> {
        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        let mut mem_cache = self.memory_cache.lock().await;
        if let Some(entry) = mem_cache.get(key) {
            if !self.is_expired(&entry.created_at) {
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                self.stats.misses.fetch_sub(1, Ordering::Relaxed);
                self.stats.data_served_from_cache_bytes.fetch_add(entry.data.len() as u64, Ordering::Relaxed);
                debug!("CACHE HIT (memory): {}", key);
                return Some(entry.clone());
            } else {
                mem_cache.pop(key);
            }
        }
        drop(mem_cache);

        if let Some(path) = self.key_to_path(key) {
            if let Ok(file_content) = tokio::fs::read(&path).await {
                if let Ok((disk_entry, data)) = bincode::deserialize::<(CacheEntryDisk, Vec<u8>)>(&file_content) {
                    if !self.is_expired(&disk_entry.created_at) {
                        let entry_data_len = data.len() as u64;
                        let entry = Arc::new(CacheEntry {
                            headers: disk_entry.headers,
                            created_at: disk_entry.created_at,
                            url: disk_entry.url,
                            data,
                        });
                        self.stats.hits.fetch_add(1, Ordering::Relaxed);
                        self.stats.misses.fetch_sub(1, Ordering::Relaxed);
                        self.stats.data_served_from_cache_bytes.fetch_add(entry_data_len, Ordering::Relaxed);
                        debug!("CACHE HIT (disk): {}", key);
                        let mut mem_cache = self.memory_cache.lock().await;
                        mem_cache.put(key.to_string(), entry.clone());
                        return Some(entry);
                    } else {
                        let _ = tokio::fs::remove_file(path).await;
                    }
                }
            }
        }
        debug!("CACHE MISS: {}", key);
        None
    }
    
    pub async fn put(&self, key: &str, url: &str, data: Vec<u8>, headers: HeaderMap) {
        let entry = Arc::new(CacheEntry {
            data,
            headers,
            created_at: Utc::now(),
            url: url.to_string(),
        });

        if let Some(path) = self.key_to_path(key) {
            let disk_entry = CacheEntryDisk {
                headers: entry.headers.clone(),
                created_at: entry.created_at,
                url: entry.url.clone(),
            };
            if let Ok(encoded) = bincode::serialize(&(&disk_entry, &entry.data)) {
                let file_existed = path.exists();
                let original_size = if file_existed { fs::metadata(&path).map(|m| m.len()).unwrap_or(0) } else { 0 };

                if tokio::fs::write(&path, &encoded).await.is_ok() {
                    let new_size = encoded.len() as u64;
                    if !file_existed {
                        self.stats.disk_items.fetch_add(1, Ordering::Relaxed);
                    }
                    // More accurate size calculation
                    self.stats.total_disk_size_bytes.fetch_add(new_size, Ordering::Relaxed);
                    self.stats.total_disk_size_bytes.fetch_sub(original_size, Ordering::Relaxed);

                }
            } else {
                warn!("Failed to serialize cache entry for key: {}", key);
            }
        }
        
        let mut mem_cache = self.memory_cache.lock().await;
        mem_cache.put(key.to_string(), entry);
    }

    pub async fn clear(&self) {
        self.memory_cache.lock().await.clear();
        if let Some(path) = &self.disk_path {
             if path.exists() {
                 if let Err(e) = fs::remove_dir_all(path) {
                    warn!("Could not clear disk cache (might be busy): {}", e);
                }
            }
            let _ = fs::create_dir_all(path);
        }
        self.stats.hits.store(0, Ordering::Relaxed);
        self.stats.misses.store(0, Ordering::Relaxed);
        self.stats.disk_items.store(0, Ordering::Relaxed);
        self.stats.total_disk_size_bytes.store(0, Ordering::Relaxed);
        self.stats.data_served_from_cache_bytes.store(0, Ordering::Relaxed);
        info!("Cache cleared successfully.");
    }
    
    pub async fn get_stats(&self) -> CacheStats {
        let hits = self.stats.hits.load(Ordering::Relaxed);
        let misses = self.stats.misses.load(Ordering::Relaxed);
        CacheStats {
            hits,
            misses,
            total_requests: hits + misses,
            in_memory_items: self.memory_cache.lock().await.len(),
            disk_items: self.stats.disk_items.load(Ordering::Relaxed),
            total_disk_size_bytes: self.stats.total_disk_size_bytes.load(Ordering::Relaxed),
            data_served_from_cache_bytes: self.stats.data_served_from_cache_bytes.load(Ordering::Relaxed),
        }
    }

    // --- BU FONKSİYON GÜNCELLENDİ ---
    pub async fn get_all_entries_metadata(&self) -> Result<Vec<CacheEntryMetadata>> {
        let mut all_entries = Vec::new();
        if let Some(path) = &self.disk_path {
            if !path.exists() { return Ok(all_entries); }
            let mut read_dir = tokio::fs::read_dir(path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                if entry.metadata().await?.is_file() {
                    let file_path = entry.path();
                    if let Ok(file_content) = tokio::fs::read(&file_path).await {
                        match bincode::deserialize::<(CacheEntryDisk, Vec<u8>)>(&file_content) {
                            Ok((disk_entry, data)) => {
                                let hash = file_path.file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or_default()
                                    .to_string();

                                all_entries.push(CacheEntryMetadata {
                                    headers: disk_entry.headers,
                                    created_at: disk_entry.created_at,
                                    url: disk_entry.url,
                                    size: data.len() as u64,
                                    hash,
                                });
                            },
                            Err(e) => {
                                warn!("[Cache Listing] Failed to deserialize entry: {:?}, Error: {}", file_path, e);
                            }
                        }
                    } else {
                        warn!("[Cache Listing] Failed to read cache file: {:?}", file_path);
                    }
                }
            }
        }
        // Girdileri en yeniye göre sıralayarak UI'da daha kullanışlı bir görünüm sağlayalım.
        all_entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(all_entries)
    }

    pub async fn delete_entry(&self, key_hash: &str) -> Result<()> {
        let key_to_remove_from_mem = {
            let cache = self.memory_cache.lock().await;
            cache.iter().find(|(k, _)| self.key_to_hash(k) == key_hash)
                 .map(|(k, _)| k.clone())
        };

        if let Some(key) = key_to_remove_from_mem {
            self.memory_cache.lock().await.pop(&key);
        }
        
        if let Some(path) = self.disk_path.as_ref().map(|p| p.join(key_hash)) {
            if path.exists() {
                let size = fs::metadata(&path)?.len();
                tokio::fs::remove_file(&path).await?;
                self.stats.disk_items.fetch_sub(1, Ordering::Relaxed);
                self.stats.total_disk_size_bytes.fetch_sub(size, Ordering::Relaxed);
            }
        }
        info!("Cache entry deleted: {}", key_hash);
        Ok(())
    }
}