use hadamard_core::HadamardEngine;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::ffi::CStr;
use std::os::raw::c_char;

// Global single-instance synchronization container for the quantum engine inside the Postgres memory space
static GLOBAL_HADAMARD_INSTANCE: OnceLock<HadamardEngine> = OnceLock::new();

/// Internal utility to retrieve or safely allocate the global shared quantum engine context
fn get_engine_context() -> &'static HadamardEngine {
    GLOBAL_HADAMARD_INSTANCE.get_or_init(|| {
        let shard_capacity_bytes = 4_194_304; // High-throughput 4MB shard size configuration for database tables
        let max_qubits = 29;
        HadamardEngine::new(shard_capacity_bytes, max_qubits)
    })
}

/// PostgreSQL Custom Extension C-ABI Interface entry point.
/// Simulates a native PostgreSQL User-Defined Function (UDF) or SPI link.
/// This acts as the direct synchronization bridge for target tables.
#[no_mangle]
pub unsafe extern "C" fn pg_hadamard_sync_table(
    c_collection_name: *const c_char,
    c_json_table_payload: *const c_char,
) -> i32 {
    if c_collection_name.is_null() || c_json_table_payload.is_null() {
        return -1; // Standard PostgreSQL error state marker
    }

    // Convert raw Postgres memory string pointers safely into active Rust string slices
    let collection_name = match CStr::from_ptr(c_collection_name).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let json_payload = match CStr::from_ptr(c_json_table_payload).to_str() {
        Ok(s) => s,
        Err(_) => return -3,
    };

    // Parse the high-density relational row snapshot arriving from Postgres internal tables
    let records: Vec<HashMap<String, String>> = match serde_json::from_str(json_payload) {
        Ok(data) => data,
        Err(_) => return -4,
    };

    let engine = get_engine_context();
    let total_records_count = records.len();

    // Spawning a standalone isolated Tokio blocking container thread to preserve Postgres scheduler bounds
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build() {
            Ok(rt) => rt,
            Err(_) => return -5,
        };

    let sync_result = runtime.block_on(async {
        engine.load_dataset(collection_name, records).await
    });

    match sync_result {
        Ok(_) => total_records_count as i32, // Return total synchronized rows directly back to Postgres executor logs
        Err(_) => -6,
    }
}

/// Executes a sub-linear quantum query over the cached Postgres relational shards.
/// Returns a flattened JSON string block mapped straight to Postgres text registers.
#[no_mangle]
pub unsafe extern "C" fn pg_hadamard_quantum_search(
    c_sql_statement: *const c_char,
) -> *mut c_char {
    if c_sql_statement.is_null() {
        return std::ptr::null_mut();
    }

    let sql_statement = match CStr::from_ptr(c_sql_statement).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let engine = get_engine_context();
    
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build() {
            Ok(rt) => rt,
            Err(_) => return std::ptr::null_mut(),
        };

    let query_result = runtime.block_on(async {
        engine.query(sql_statement).await
    });

    match query_result {
        Ok(resultSet) => {
            // Serialize structural quantum hits back into standard Postgres JSON text compliant strings
            if let Ok(json_string) = serde_json::to_string(&resultSet) {
                // Allocate memory using standard C allocator or libc to hand pointer ownership safely to Postgres backend
                let c_string = std::ffi::CString::new(json_string).unwrap();
                c_string.into_raw()
            } else {
                std::ptr::null_mut()
            }
        }
        Err(_) => std::ptr::null_mut(),
    }
}
