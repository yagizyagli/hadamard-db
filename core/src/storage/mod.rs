pub mod qram;
pub mod cache;

#[derive(Debug, Clone)]
pub struct StorageEngineConfig {
    pub shard_max_bytes: usize,
    pub cache_max_bytes: usize,
    pub enforce_pqc_encryption: bool,
}
