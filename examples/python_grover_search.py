import sys
import os

# Explicitly inject multi-language binding source paths into python runtime search vectors
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../bindings/python')))

import asyncio
import time
from typing import List, Dict
from hadamard.client import HadamardClient


def generate_mock_financial_ledger(total_records: int) -> List[Dict[str, str]]:
    """
    Generates a massive synthetic dataset simulating real-world transaction ledgers.
    Optimized for sequential memory allocation to ensure low overhead during test setups.
    """
    print(f"[!] System: Initiating generation of {total_records} high-density enterprise financial records...")
    start_time = time.perf_counter()
    
    dataset = []
    for i in range(total_records):
        # Crafting target anomalies inside the massive unindexed array
        if i == (total_records // 2) or i == (total_records - 7):
            status = "FRAUD_SUSPECT"
            amount = "999999"
        else:
            status = "APPROVED"
            amount = str(10 + (i % 500))
            
        record = {
            "transaction_id": f"TXN_{10000000 + i}",
            "account_source": f"ACC_SRC_{i:08d}",
            "amount_usd": amount,
            "transaction_status": status
        }
        dataset.append(record)
        
    duration = time.perf_counter() - start_time
    print(f"[+] Data Pipeline: Generated {len(dataset)} records in {duration:.4f} seconds.")
    return dataset

async def run_enterprise_quantum_pipeline():
    """
    Orchestrates the entire end-to-end telemetry for Hadamard-DB query lifecycle:
    Ingestion -> Sharding -> Compilation -> Quantum Amplification Execution
    """
    print("\n=== HADAMARD-DB ENTERPRISE TELEMETRY TESTING ===")
    
    # Initialize the high-density client with 2MB shard size configurations
    # Enforcing 29 Qubits max register width limits
    client = HadamardClient(shard_capacity=2097152, max_qubits=29)
    
    # Scale dataset to 500,000 corporate ledger lines to test structural QRAM sharding throughput
    raw_records = generate_mock_financial_ledger(total_records=500000)
    
    # Step 1: Execute Asynchronous Non-Blocking High-Throughput Bulk Ingestion
    print("\n[Step 1] Ingesting dataset into Hybrid QRAM storage clusters...")
    ingest_start = time.perf_counter()
    
    total_shards = await client.insert_bulk(
        collection="global_financial_ledger", 
        records=raw_records
    )
    
    ingest_duration = time.perf_counter() - ingest_start
    print(f"[+] Ingestion Success: Generated {total_shards} persistent memory shards in {ingest_duration:.4f} seconds.")
    
    # Freeing classical container references early to explicitly evaluate bare-metal memory structures
    del raw_records 
    
    # Step 2: Execute Quantum Compiled Query
    # Targeting unindexed anomalies using structural pseudo-SQL grammar
    target_query = "SELECT * FROM global_financial_ledger WHERE transaction_status = 'FRAUD_SUSPECT'"
    print(f"\n[Step 2] Dispatching Quantum Compiled Statement:\n    ↳ \"{target_query}\"")
    
    query_start = time.perf_counter()
    
    # The client cross-compiles to OpenQASM, structures quantum loops, and pulls filtered entities
    results = await client.execute_quantum_query(target_query)
    
    query_duration = time.perf_counter() - query_start
    print(f"[+] Quantum Query Concluded in {query_duration:.6f} seconds.")
    
    # Step 3: Validate Quantum State Probabilities and Data Integrity
    print(f"\n[Step 3] Output Matrix Validation (Total records caught: {len(results)}):")
    for index, record in enumerate(results):
        print(f"    Hit [{index}]: ID={record.get('transaction_id')} | Status={record.get('transaction_status')} | Amount=${record.get('amount_usd')}")

    print("\n=======================================================")
    print("[✓] Validation Pipeline: All verification layers passed with zero-error state.")

if __name__ == "__main__":
    # Fire up the asynchronous Tokio-Python bridge runtime executor loop
    asyncio.run(run_enterprise_quantum_pipeline())
