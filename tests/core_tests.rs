use hadamard_core::HadamardEngine;
use hadamard_core::compiler::parser::QuantumCompiler;
use std::collections::HashMap;

#[tokio::test]
async fn test_monolithic_engine_ingestion_and_sharding_boundaries() {
    // Initialize engine with explicit 100 bytes micro-shard constraints to forcefully trigger multi-sharding loops
    let engine = HadamardEngine::new(100, 29);
    
    let mut dataset = Vec::new();
    for i in 0..10 {
        let mut row = HashMap::new();
        row.insert("record_id".to_string(), format!("ID_{:04}", i));
        row.insert("payload_key".to_string(), "LARGE_DENSE_STREAMING_TEST_VALUE_METRIC_DATA".to_string());
        dataset.push(row);
    }

    let result = engine.load_dataset("boundary_test_workspace", dataset).await;
    
    assert!(result.is_ok(), "Storage tier failed during strict boundary sharding execution.");
    let total_shards = result.unwrap();
    assert!(total_shards > 1, "Engine failed to split data into multiple independent memory shards.");
}

#[tokio::test]
async fn test_compiler_parser_sql_grammar_integrity() {
    let valid_query = "SELECT * FROM users WHERE status = 'ACTIVE'";
    let parse_result = QuantumCompiler::parse_query(valid_query);
    
    assert!(parse_result.is_ok(), "Compiler failed to parse valid compliant SQL token sequences.");
    let structured_query = parse_result.unwrap();
    assert_eq!(structured_query.target_collection, "users");
    assert_eq!(structured_query.field, "status");
    assert_eq!(structured_query.value, "ACTIVE");

    // Test adversarial invalid syntax injections
    let invalid_query = "SELECT STUFF FROM users";
    let invalid_parse_result = QuantumCompiler::parse_query(invalid_query);
    assert!(invalid_parse_result.is_err(), "Parser failed to catch illegal structural grammar breaches.");
}

#[tokio::test]
async fn test_asynchronous_parallel_query_contention() {
    let engine = std::sync::Arc::new(HadamardEngine::new(1024, 29));
    
    // Warm up workspace clusters
    let mut data = Vec::new();
    let mut row = HashMap::new();
    row.insert("node_id".to_string(), "NODE_CORE_77".to_string());
    row.insert("telemetry".to_string(), "CRITICAL".to_string());
    data.push(row);
    engine.load_dataset("concurrent_workspace", data).await.unwrap();

    let mut task_handles = Vec::new();

    // Spawn 20 highly concurrent parallel async threads hitting the same memory lock structure simultaneously
    for _ in 0..20 {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            let statement = "SELECT * FROM concurrent_workspace WHERE telemetry = 'CRITICAL'";
            engine_clone.query(statement).await
        });
        task_handles.push(handle);
    }

    for handle in task_handles {
        let thread_execution_result = handle.await;
        assert!(thread_execution_result.is_ok(), "Tokio task execution worker panicked under load.");
        let query_result = thread_execution_result.unwrap();
        assert!(query_result.is_ok(), "HadamardEngine internal locks faulted under active parallel contention loops.");
    }
}
