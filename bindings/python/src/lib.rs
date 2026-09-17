use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3_asyncio::tokio::future_into_py;
use hadamard_core::HadamardEngine;
use std::collections::HashMap;
use std::sync::Arc;

/// Python-facing wrapper for the high-performance Rust HadamardEngine.
/// Manages thread-safe pointer references across the Python GIL boundary.
#[pyclass]
pub struct PyHadamardEngine {
    pub inner: Arc<HadamardEngine>,
}

#[pymethods]
impl PyHadamardEngine {
    #[new]
    pub fn new(shard_capacity: usize, max_qubits_supported: usize) -> Self {
        Self {
            inner: Arc::new(HadamardEngine::new(shard_capacity, max_qubits_supported)),
        }
    }

    /// Asynchronously ingests a large block of Python dictionary datasets into Rust QRAM Shards.
    /// Returns a Python coroutine compatible with `await`.
    pub fn load_dataset<'p>(
        &self,
        py: Python<'p>,
        collection_name: String,
        dataset: Vec<HashMap<String, String>>,
    ) -> PyResult<&'p PyAny> {
        let engine_ref = self.inner.clone();
        
        // Marshal execution flow directly into the background Tokio runtime pool
        future_into_py(py, async move {
            engine_ref.load_dataset(&collection_name, dataset)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Ingestion crashed: {}", e)))
        })
    }

    /// Compiles and dispatches a full Quantum Query pipeline from Python context.
    /// Extracts probabilistic results back into pure native Python dictionaries.
    pub fn query<'p>(&self, py: Python<'p>, raw_sql_query: String) -> PyResult<&'p PyAny> {
        let engine_ref = self.inner.clone();
        
        future_into_py(py, async move {
            engine_ref.query(&raw_sql_query)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Quantum execution query failed: {}", e)))
        })
    }
}

/// The actual entry point mapping for the native compiled Python extension module.
#[pymodule]
fn _hadamard_core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyHadamardEngine>()?;
    Ok(())
}
