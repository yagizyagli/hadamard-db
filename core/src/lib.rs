pub mod compiler;
pub mod algorithms;
pub mod storage;

use crate::compiler::parser::QuantumCompiler;
use crate::algorithms::grover::GroverSearchEngine;
use crate::storage::qram::{HybridQramStorage, StorageError};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Compiler tier malfunction: {0}")]
    CompilerFailure(#[from] crate::compiler::parser::CompilerError),
    #[error("Storage tier malfunction: {0}")]
    StorageFailure(#[from] crate::storage::qram::StorageError),
    #[error("Quantum algorithm pipeline failure: {0}")]
    ExecutionFailure(#[from] crate::algorithms::grover::QuantumEngineError),
}

pub struct HadamardEngine {
    pub storage: Arc<HybridQramStorage>,
    pub quantum_runtime: Arc<GroverSearchEngine>,
}

impl HadamardEngine {
    pub fn new(shard_capacity: usize, max_qubits_supported: usize) -> Self {
        Self {
            storage: Arc::new(HybridQramStorage::new(shard_capacity)),
            quantum_runtime: Arc::new(GroverSearchEngine::new(max_qubits_supported)),
        }
    }

    pub async fn load_dataset(
        &self,
        collection_name: &str,
        dataset: Vec<HashMap<String, String>>,
    ) -> Result<usize, EngineError> {
        let total_shards = self.storage.ingest_and_shard(collection_name, dataset).await?;
        Ok(total_shards)
    }

    pub async fn query(
        &self,
        raw_sql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, EngineError> {
        let structured_query = QuantumCompiler::parse_query(raw_sql_query)?;
        let circuit_plan = QuantumCompiler::compile_to_circuit(&structured_query);

        let shards = self.storage.fetch_shard_buffers(&structured_query.target_collection).await?;
        
        let mut universal_dataset = Vec::new();
        for shard in shards {
            let content_str = String::from_utf8_lossy(&shard.dense_buffer);
            let raw_records: Vec<&str> = content_str.split('\u{001E}').collect();
            
            for raw_record in raw_records {
                if raw_record.is_empty() { continue; }
                let mut record_map = HashMap::new();
                let pairs: Vec<&str> = raw_record.split('\u{001F}').collect();
                
                for chunk in pairs.chunks_exact(2) {
                    if chunk.len() == 2 {
                        record_map.insert(chunk[0].to_string(), chunk[1].to_string());
                    }
                }
                if !record_map.is_empty() {
                    universal_dataset.push(record_map);
                }
            }
        }

        let hit_indices = self.quantum_runtime.execute_search(
            &circuit_plan,
            &universal_dataset,
            &structured_query.field,
            &structured_query.value,
        )?;

        let mut final_results = Vec::with_capacity(hit_indices.len());
        for idx in hit_indices {
            if let Some(record) = universal_dataset.get(idx) {
                final_results.push(record.clone());
            }
        }

        Ok(final_results)
    }
}
