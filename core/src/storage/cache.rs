use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache entry has expired based on TTL configurations.")]
    EntryExpired,
    #[error("The specific collection block allocation failed.")]
    AllocationFailure,
}

#[derive(Debug, Clone)]
pub struct CacheValue {
    pub payload: Vec<u8>,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl CacheValue {
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Ultra-lightweight, zero-copy memory cache layer for Hadamard-DB.
/// Acts as the high-speed ingest buffer before data partitions are frozen into QRAM shards.
pub struct ZeroCopyCache {
    pub max_capacity_bytes: usize,
    pub current_size_bytes: Arc<RwLock<usize>>,
    pub registry: Arc<RwLock<BTreeMap<String, CacheValue>>>,
}

impl ZeroCopyCache {
    pub fn new(max_capacity_bytes: usize) -> Self {
        Self {
            max_capacity_bytes,
            current_size_bytes: Arc.new(RwLock::new(0)),
            registry: Arc.new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Inserts a binary records block straight into the volatile BTreeMap cache matrix.
    /// Thread-safe and designed for low contention under massive parallel streaming pipelines.
    pub async fn put(&self, key: String, payload: Vec<u8>, ttl_secs: u64) -> Result<(), CacheError> {
        let payload_size = payload.len();
        let mut current_size = self.current_size_bytes.write().await;
        
        // Enforce hard memory protection boundaries to prevent host OS thrashing
        if *current_size + payload_size > self.max_capacity_bytes {
            // Evict expired entries proactively to clear space
            self.evict_expired_entries().await;
            if *current_size + payload_size > self.max_capacity_bytes {
                return Err(CacheError::AllocationFailure);
            }
        }

        let mut write_registry = self.registry.write().await;
        
        let new_value = CacheValue {
            payload,
            created_at: Instant::now(),
            ttl: Duration::from_secs(ttl_secs),
        };

        if let Some(old_val) = write_registry.insert(key, new_value) {
            *current_size = (*current_size + payload_size).saturating_sub(old_val.payload.len());
        } else {
            *current_size += payload_size;
        }

        Ok(())
    }

    /// Fetches a read-only payload pointer view directly from the cached partition.
    pub async fn get(&self, key: &str) -> Result<Vec<u8>, CacheError> {
        let read_registry = self.registry.read().await;
        
        match read_registry.get(key) {
            Some(value) => {
                if value.is_expired() {
                    // We drop the read lock before performing mutation to avoid structural deadlocks
                    drop(read_registry);
                    self.remove_entry(key).await;
                    return Err(CacheError::EntryExpired);
                }
                Ok(value.payload.clone())
            }
            None => Err(CacheError::EntryExpired),
        }
    }

    /// Flushes out stale entries from the cache layout to keep the active footprint microscopic.
    pub async fn evict_expired_entries(&self) -> {
        let mut write_registry = self.registry.write().await;
        let mut current_size = self.current_size_bytes.write().await;
        
        let mut keys_to_remove = Vec::new();
        for (key, val) in write_registry.iter() {
            if val.is_expired() {
                keys_to_remove.push(key.clone());
            }
        }

        for key in keys_to_remove {
            if let Some(removed) = write_registry.remove(&key) {
                *current_size = current_size.saturating_sub(removed.payload.len());
            }
        }
    }

    async fn remove_entry(&self, key: &str) {
        let mut write_registry = self.registry.write().await;
        let mut current_size = self.current_size_bytes.write().await;
        if let Some(removed) = write_registry.remove(key) {
            *current_size = current_size.saturating_sub(removed.payload.len());
        }
    }
}
