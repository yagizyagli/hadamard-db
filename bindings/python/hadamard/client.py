import sys
import os

# 1. ALWAYS RESOLVE WORKSPACE PATHS FIRST
CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
WORKSPACE_ROOT = os.path.abspath(os.path.join(CURRENT_DIR, "../../../"))

# 2. FORCEFULLY INJECT ALL CORE TARGET RELEASES TO THE ABSOLUTE FRONT OF SEARCH PATHS
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "target/release/maturin"))
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "target/release"))
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "core/target/release"))
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "bindings/python"))

# 3. CRITICAL CRATE PURGE: Evict and purge the stale broken cache allocation out of Python's memory registers
if "hadamard_core" in sys.modules:
    del sys.modules["hadamard_core"]

import asyncio
from typing import List, Dict, Any

# 4. FORCE RE-IMPORT: Directly bind onto the fresh workspace-compiled dynamic library target
import hadamard_core
from hadamard_core import HadamardEngine as NativeEngine

class HadamardClient:
    """
    Enterprise High-Fidelity Client SDK for Hadamard-DB.
    Combines the safety of Python type systems with the raw bare-metal execution 
    speed of the underlying Rust quantum compilation cluster.
    """
    def __init__(self, shard_capacity: int = 1048576, max_qubits: int = 29):
        """
        Initializes the quantum data engine.
        :param shard_capacity: Max bytes allocated per volatile QRAM shard. Default 1MB.
        :param max_qubits: Maximum width of the destination quantum processor register.
        """
        self._engine = NativeEngine(shard_capacity, max_qubits)

    async def insert_bulk(self, collection: str, records: List[Dict[str, str]]) -> int:
        """
        Pushes massive array payloads straight down to local NVMe memory chunks, 
        building the target qubit indices instantly.
        :param collection: Target workspace or database cluster partition.
        :param records: List of string-to-string dictionary schemas.
        :return: Total number of physical quantum storage shards generated.
        """
        if not collection or not records:
            raise ValueError("Collection name and dataset payload cannot be empty.")
        
        # Crosses the C-ABI border asynchronously without blocking the Python event loop
        total_shards: int = await self._engine.load_dataset(collection, records)
        return total_shards

    async def execute_quantum_query(self, sql_query: str) -> List[Dict[str, str]]:
        """
        Compiles structural query syntax, translates it into OpenQASM primitives,
        runs quantum search over Shards, and yields results.
        :param sql_query: Targeted query statement (e.g. 'SELECT * FROM users WHERE status = "active"')
        :return: High-probability search results matching the matrix oracle state.
        """
        if not sql_query:
            raise ValueError("SQL string sequence cannot be empty.")
            
        raw_results: List[Dict[str, str]] = await self._engine.query(sql_query)
        return raw_results
