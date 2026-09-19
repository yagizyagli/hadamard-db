use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use atomic_counter::{AtomicCounter, ConsistentCounter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Shard allocation failed due to memory exhaustion.")]
    MemoryExhaustion,
    #[error("The requested key partition '{0}' does not exist in QRAM index.")]
    PartitionNotFound(String),
    #[error("Data corruption detected during quantum state preparation: {0}")]
    Corruption(String),
}

#[derive(Debug, Clone)]
pub struct QuantumShard {
    pub shard_id: u64,
    pub address_qubit_mapping: HashMap<String, usize>,
    pub dense_buffer: Vec<u8>,
}

pub struct HybridQramStorage {
    pub shard_capacity: usize,
    pub active_shards: Arc<RwLock<HashMap<String, Vec<QuantumShard>>>>,
    pub global_transaction_counter: ConsistentCounter,
}

impl HybridQramStorage {
    pub fn new(shard_capacity: usize) -> Self {
        Self {
            shard_capacity,
            active_shards: Arc::new(RwLock::new(HashMap::new())),
            global_transaction_counter: ConsistentCounter::new(0),
        }
    }

    pub async fn ingest_and_shard(
        &self,
        collection_name: &str,
        records: Vec<HashMap<String, String>>,
    ) -> Result<usize, StorageError> {
        let mut shard_map = self.active_shards.write().await;
        let collection_entry = shard_map.entry(collection_name.to_string()).or_insert_with(Vec::new);

        let mut current_shard_id = self.global_transaction_counter.inc() as u64;
        let mut current_buffer = Vec::new();
        let mut current_mapping = HashMap::new();
        let mut qubit_cursor = 0;
        let mut total_shards_created = 0;

        for record in records {
            let mut serialized_record = Vec::new();
            for (key, val) in &record {
                serialized_record.extend_from_slice(key.as_bytes());
                serialized_record.push(0x1F); 
                serialized_record.extend_from_slice(val.as_bytes());
                serialized_record.push(0x1E); 
                
                if !current_mapping.contains_key(key) {
                    current_mapping.insert(key.clone(), qubit_cursor);
                    qubit_cursor += 1;
                }
            }

            current_buffer.extend(serialized_record);

            if current_buffer.len() >= self.shard_capacity {
                collection_entry.push(QuantumShard {
                    shard_id: current_shard_id,
                    address_qubit_mapping: current_mapping.clone(),
                    dense_buffer: current_buffer.clone(),
                });
                
                total_shards_created += 1;
                current_shard_id = self.global_transaction_counter.inc() as u64;
                current_buffer.clear();
                current_mapping.clear();
                qubit_cursor = 0;
            }
        }

        if !current_buffer.is_empty() {
            collection_entry.push(QuantumShard {
                shard_id: current_shard_id,
                address_qubit_mapping: current_mapping,
                dense_buffer: current_buffer,
            });
            total_shards_created += 1;
        }

        Ok(total_shards_created)
    }

    pub async fn fetch_shard_buffers(
        &self,
        collection_name: &str,
    ) -> Result<Vec<QuantumShard>, StorageError> {
        let shard_map = self.active_shards.read().await;
        match shard_map.get(collection_name) {
            Some(shards) => Ok(shards.clone()),
            None => Err(StorageError::PartitionNotFound(collection_name.to_string())),
        }
    }
}
