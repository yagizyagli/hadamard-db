pub mod compiler;
pub mod algorithms;
pub mod storage;

use crate::compiler::QuantumCompiler;
use crate::algorithms::grover::GroverSearchEngine;
use crate::storage::qram::HybridQramStorage;
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

/// The monolithic orchestral entry point for Hadamard-DB operations.
/// Thread-safe, non-blocking, and built to handle enterprise Big Data ingestion
/// concurrently with live quantum computation pipelines.
pub struct HadamardEngine {
    storage: Arc<HybridQramStorage>,
    quantum_runtime: Arc<GroverSearchEngine>,
}

impl HadamardEngine {
    pub fn new(shard_capacity: usize, max_qubits_supported: usize) -> Self {
        Self {
            storage: Arc::new(HybridQramStorage::new(shard_capacity)),
            quantum_runtime: Arc::new(GroverSearchEngine::new(max_qubits_supported)),
        }
    }

    /// Exposes a secure, high-throughput ingest port to append datasets to memory spaces
    pub async fn load_dataset(
        &self,
        collection_name: &str,
        dataset: Vec<HashMap<String, String>>,
    ) -> Result<usize, EngineError> {
        let total_shards = self.storage.ingest_and_shard(collection_name, dataset).await?;
        Ok(total_shards)
    }

    /// Executes an end-to-end Quantum Query Workflow:
    /// 1. Compiles classical SQL syntax down to a concrete mathematical Circuit Plan.
    /// 2. Fetches binary dense shards from the volatile hybrid cache layer.
    /// 3. Fires up the quantum computing amplification sequence via the internal QPU pipeline.
    pub async fn query(
        &self,
        raw_sql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, EngineError> {
        // Step 1: Compilation
        let structured_query = QuantumCompiler::parse_query(raw_sql_query)?;
        let circuit_plan = QuantumCompiler::compile_to_circuit(&structured_query);

        // Step 2: Extract low-overhead storage views
        let shards = self.storage.fetch_shard_buffers(&structured_query.target_collection).await?;
        
        // Reconstruct records dynamically for deep evaluation inside the processing loop
        let mut universal_dataset = Vec::new();
        for shard in shards {
            let content_str = String::from_utf8_lossy(&shard.dense_buffer);
            let raw_records: Vec<&str> = content_str.split('\u{001E}').collect();
            
            for raw_record in raw_records {
                if raw_record.is_empty() { continue; }
                let mut record_map = HashMap::new();
                let pairs: Vec<&str> = raw_record.split('\u{001F}').collect();
                
                // Pair reconstructor loop
                for chunk in pairs.chunks_exact(2) {
                    record_map.insert(chunk[0].to_string(), chunk[1].to_string());
                }
                if !record_map.is_empty() {
                    universal_dataset.push(record_map);
                }
            }
        }

        // Step 3: Run Quantum Amplitude Amplification Sequence
        let hit_indices = self.quantum_runtime.execute_search(
            &circuit_plan,
            &universal_dataset,
            &structured_query.field,
            &structured_query.value,
        )?;

        // Map quantum state hits back to structural business entities
        let mut final_results = Vec::with_capacity(hit_indices.len());
        for idx in hit_indices {
            if let Some(record) = universal_dataset.get(idx) {
                final_results.push(record.clone());
            }
        }

        Ok(final_results)
    }
}
