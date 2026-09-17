pub mod parser;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryOperator {
    Equal,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone)]
pub struct QuantumQuery {
    pub target_collection: String,
    pub field: String,
    pub operator: QueryOperator,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum QuantumGate {
    H(usize),          // Hadamard Gate applied to qubit index
    X(usize),          // Pauli-X (NOT) Gate
    CX(usize, usize),  // Controlled-NOT (CNOT)
    Oracle(String),    // Quantum Oracle configured for the specific search target
}

#[derive(Debug, Clone)]
pub struct QuantumCircuitPlan {
    pub required_qubits: usize,
    pub execution_gates: Vec<QuantumGate>,
}
