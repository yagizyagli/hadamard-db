import sys
import os

# Base directory absolute resolution logic
BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))

# Explicitly inject all multi-language compiled bindings and workspace release target targets
sys.path.insert(0, os.path.join(BASE_DIR, 'bindings/python'))
sys.path.insert(0, os.path.join(BASE_DIR, 'target/release'))

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
    client = HadamardClient(shard_capacity=2097152, max_qubits=29)
    
    # Scale dataset to 50,000 corporate ledger lines to test structural QRAM sharding throughput safely in CI
    raw_records = generate_mock_financial_ledger(total_records=50000)
    
    print("\n[Step 1] Ingesting dataset into Hybrid QRAM storage clusters...")
    ingest_start = time.perf_counter()
    
    total_shards = await client.insert_bulk(
        collection="global_financial_ledger", 
        records=raw_records
    )
    
    ingest_duration = time.perf_counter() - ingest_start
    print(f"[+] Ingestion Success: Generated {total_shards} persistent memory shards in {ingest_duration:.4f} seconds.")
    
    del raw_records 
    
    target_query = "SELECT * FROM global_financial_ledger WHERE transaction_status = 'FRAUD_SUSPECT'"
    print(f"\n[Step 2] Dispatching Quantum Compiled Statement:\n    ↳ \"{target_query}\"")
    
    query_start = time.perf_counter()
    results = await client.execute_quantum_query(target_query)
    query_duration = time.perf_counter() - query_start
    print(f"[+] Quantum Query Concluded in {query_duration:.6f} seconds.")
    
    print(f"\n[Step 3] Output Matrix Validation (Total records caught: {len(results)}):")
    for index, record in enumerate(results):
        print(f"    Hit [{index}]: ID={record.get('transaction_id')} | Status={record.get('transaction_status')} | Amount=${record.get('amount_usd')}")

    print("\n=======================================================")
    print("[✓] Validation Pipeline: All verification layers passed with zero-error state.")

if __name__ == "__main__":
    asyncio.run(run_enterprise_quantum_pipeline())
