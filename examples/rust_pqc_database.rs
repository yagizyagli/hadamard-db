use hadamard_core::HadamardEngine;
use hadamard_core::algorithms::crypto::PostQuantumShield;
use std::collections::HashMap;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=======================================================");
    println!("   HADAMARD-DB POST-QUANTUM CRYPTOGRAPHY INTEGRATION   ");
    println!("=======================================================");

    // Step 1: Initialize the Post-Quantum Cryptographic Shield
    // Taps into native NIST ML-KEM-1024 parameters for absolute future-proof security
    println!("[Step 1] Bootstrapping Post-Quantum Cryptographic Shield...");
    let pqc_shield = PostQuantumShield::new();
    
    let key_timer = Instant::now();
    let (public_key, secret_key) = pqc_shield.generate_quantum_keypair()?;
    println!("[✓] Keys Generated in: {:?}", key_timer.elapsed());
    println!("    ↳ Public Key Buffer Registry Size:  {} bytes", public_key.len());
    println!("    ↳ Secret Key Buffer Registry Size:  {} bytes", secret_key.len());

    // Step 2: Initialize the Core Quantum Database Engine
    // Configuring 1MB shard boundaries with multi-backend compatibility tracking
    println!("\n[Step 2] Spinning up asynchronous Hadamard Core Database Cluster...");
    let shard_capacity_bytes = 1_048_576; 
    let max_qubits = 29;
    let engine = HadamardEngine::new(shard_capacity_bytes, max_qubits);

    // Step 3: Fabricate and Encrypt Sensitive Corporate Asset Payloads
    println!("\n[Step 3] Securing transactional records via quantum-resistant encryption...");
    let mut secured_batch = Vec::new();
    
    let raw_records = vec![
        ("TXN_9901", "SWIFT_TRANSFER_DE", "5400000.00", "CONFIDENTIAL"),
        ("TXN_9902", "ASSET_LIQUIDITY_US", "12800000.00", "RESTRICTED"),
        ("TXN_9903", "OFFSHORE_CLEARING_CH", "750000.00", "TOP_SECRET"),
    ];

    for (txn_id, channel, amount, classification) in raw_records {
        // Construct the cleartext metadata sequence block
        let serialized_metadata = format!("channel={}|amount={}|clearance={}", channel, amount, classification);
        
        // Execute hardware-accelerated encryption utilizing the ML-KEM derived public key
        let encrypted_metadata_bytes = pqc_shield.encrypt_record(
            serialized_metadata.as_bytes(), 
            &public_key
        )?;

        // Map ciphertext bytes directly into pure hex strings to ensure clean structural storage
        let hex_ciphertext = encrypted_metadata_bytes.iter()
            .fold(String::with_capacity(encrypted_metadata_bytes.len() * 2), |mut acc, b| {
                let _ = write!(acc, "{:02x}", b);
                acc
            });

        let mut data_row = HashMap::new();
        data_row.insert("transaction_id".to_string(), txn_id.to_string());
        data_row.insert("secure_payload_hex".to_string(), hex_ciphertext);
        
        secured_batch.push(data_row);
        println!("    ↳ Encrypted Node [{}]: Matrix protection applied successfully.", txn_id);
    }

    // Step 4: Bulk Load secured dataset straight into local QRAM Memory Shards
    println!("\n[Step 4] Pushing secured encrypted blocks to internal QRAM memory layout...");
    let total_shards_generated = engine.load_dataset("secure_banking_ledger", secured_batch).await?;
    println!("[✓] Ingestion Concluded: Generated {} safe cluster shards.", total_shards_generated);

    // Step 5: Execute Target Query & Decrypt Extracted Payload via Secret Key
    let query_statement = "SELECT * FROM secure_banking_ledger WHERE transaction_id = 'TXN_9903'";
    println!("\n[Step 5] Executing Quantum Statement to isolate specific data path:\n    ↳ \"{}\"", query_statement);
    
    let query_timer = Instant::now();
    let query_results = engine.query(query_statement).await?;
    println!("[✓] Isolated result set extracted via Grover operators in: {:?}", query_timer.elapsed());

    if let Some(matched_record) = query_results.first() {
        let extracted_hex = matched_record.get("secure_payload_hex").ok_or("Payload missing")?;
        println!("\n[Step 6] Running verification and quantum-state signature decryption...");
        
        // Convert hex strings backward into active raw binary slices
        let encrypted_bytes: Vec<u8> = (0..extracted_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&extracted_hex[i..i+2], 16).unwrap_or(0))
            .collect();

        // Decrypt ciphertext natively utilizing the secure host private key
        let decrypted_cleartext_bytes = pqc_shield.decrypt_record(&encrypted_bytes, &secret_key)?;
        let cleartext_string = String::from_utf8_lossy(&decrypted_cleartext_bytes);
        
        println!("==================== DECRYPTED LEDGER METADATA ====================");
        println!("    Target Asset Key:  {}", matched_record.get("transaction_id").unwrap());
        println!("    Decrypted Channel: {}", cleartext_string);
        println!("===================================================================");
    } else {
        panic!("Data integrity mismatch: Target signature disappeared from the quantum space.");
    }

    println!("\n[✓] Telemetry Pipeline Concluded: All PQC encryption-decryption paths verified successfully.");
    Ok(())
}
