import asyncio
from typing import List, Dict, Any
from ._hadamard_core import PyHadamardEngine

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
        self._engine = PyHadamardEngine(shard_capacity, max_qubits)

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
