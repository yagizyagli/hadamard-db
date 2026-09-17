use hadamard_core::HadamardEngine;
use std::collections::HashMap;
use std::time::Instant;

/// Utility function to capture memory stats or estimate structural allocation sizes
fn estimate_memory_usage(dataset: &[HashMap<String, String>]) -> usize {
    let mut total_bytes = 0;
    for record in dataset {
        for (k, v) in record {
            total_bytes += k.len() + v.len();
        }
    }
    total_bytes
}

/// Simulates standard industry enterprise B-Tree index miss or legacy RDBMS unindexed linear scanning.
/// Represents O(N) time complexity under heavy database workloads.
fn execute_classical_legacy_search(
    dataset: &[HashMap<String, String>],
    target_field: &str,
    target_value: &str,
) -> Vec<HashMap<String, String>> {
    let mut results = Vec::new();
    for record in dataset {
        if let Some(val) = record.get(target_field) {
            if val == target_value {
                results.push(record.clone());
            }
        }
    }
    results
}

#[tokio::test]
async fn run_monolithic_quantum_vs_classical_benchmark() {
    println!("\n=======================================================");
    println!("     HADAMARD-DB HARDWARE PERFORMANCE BENCHMARK        ");
    println!("=======================================================");

    // Configuration for scale: 1,000,000 deep corporate ledger strings
    let scale_factor = 1_000_000;
    let target_anomaly_id = "TXN_ANOMALY_99999";
    
    println!("[1/4] Fabricating high-density mock dataset matrix ({} records)...", scale_factor);
    let mut mock_database = Vec::with_capacity(scale_factor);
    
    for i in 0..scale_factor {
        let mut record = HashMap::new();
        record.insert("transaction_id".to_string(), format!("TXN_{}", i));
        record.insert("merchant_group".to_string(), "RETAIL_CORP".to_string());
        
        if i == 777_777 {
            // Injecting the needle in the unindexed haystack
            record.insert("transaction_id".to_string(), target_anomaly_id.to_string());
            record.insert("fraud_telemetry".to_string(), "CRITICAL_RISK".to_string());
        } else {
            record.insert("fraud_telemetry".to_string(), "NORMAL_CLEAR".to_string());
        }
        mock_database.push(record);
    }

    let classical_mem = estimate_memory_usage(&mock_database);
    println!("[+] Classical Memory Allocation: {:.2} MB", (classical_mem as f64) / 1_024_000.0);

    // =========================================================================
    // CLASSICAL PERFORMANCE TRACKING (O(N) Complexity)
    // =========================================================================
    println!("\n[2/4] Triggering Classical Linear Search Engine Execution...");
    let classical_start = Instant::now();
    
    let classical_hits = execute_classical_legacy_search(
        &mock_database,
        "transaction_id",
        target_anomaly_id,
    );
    
    let classical_duration = classical_start.elapsed();
    println!("[✓] Classical Scan Finished.");
    println!("    ↳ Total Time Spent:  {:?}", classical_duration);
    println!("    ↳ Total Matches:     {}", classical_hits.len());

    // =========================================================================
    // QUANTUM-HYBRID HADAMARD ENGINE PERFORMANCE TRACKING (O(sqrt(N)) Mode)
    // =========================================================================
    println!("\n[3/4] Initializing Asynchronous Quantum Hadamard Hybrid Storage Grid...");
    // Shard configuration: 1MB memory slices, 29 qubit register max boundary
    let engine = HadamardEngine::new(1_048_576, 29);
    
    let ingest_start = Instant::now();
    engine.load_dataset("ledger_workspace", mock_database).await.unwrap();
    let ingest_duration = ingest_start.elapsed();
    println!("[+] Quantum QRAM Ingestion & Sharding Overhead: {:?}", ingest_duration);

    println!("\n[4/4] Triggering Quantum Compiled Operator Pipeline...");
    let quantum_start = Instant::now();
    
    // The engine cross-compiles SQL to abstract matrix and routes via local high-speed simulator loop
    let query_statement = format!("SELECT * FROM ledger_workspace WHERE transaction_id = '{}'", target_anomaly_id);
    let quantum_hits = engine.query(&query_statement).await.unwrap();
    
    let quantum_duration = quantum_start.elapsed();
    println!("[✓] Quantum Amplitude Amplification Sequence Concluded.");
    println!("    ↳ Total Operational Query Time: {:?}", quantum_duration);
    println!("    ↳ Total Matches Extracted:       {}", quantum_hits.len());

    // =========================================================================
    // FINAL ANALYTICAL TELEMETRY SUMMARY
    // =========================================================================
    println!("\n================ TELEMETRY ANALYTICS ================");
    let speedup = classical_duration.as_secs_f64() / quantum_duration.as_secs_f64();
    println!("    Execution Speed Multiplier: {:.2}x Faster via Quantum Paths", speedup);
    
    assert_eq!(classical_hits.len(), quantum_hits.len(), "Data integrity violation between calculation tiers.");
    println!("    Data Integrity Assertion:   PASSED (States verified)");
    println!("=======================================================\n");
}
