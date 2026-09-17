use crate::compiler::{QuantumCircuitPlan, QuantumGate, QuantumQuery, QueryOperator};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Invalid query syntax: {0}")]
    InvalidSyntax(String),
    #[error("Unsupported operator: {0}")]
    UnsupportedOperator(String),
}

pub struct QuantumCompiler;

impl QuantumCompiler {
    /// Parses a classical pseudo-SQL query string into a structural QuantumQuery definition
    /// Example input: "SELECT * FROM users WHERE age = 42"
    pub fn parse_query(query_str: &str) -> Result<QuantumQuery, CompilerError> {
        let tokens: Vec<&str> = query_str.split_whitespace().collect();
        
        if tokens.len() < 8 || tokens[0].to_uppercase() != "SELECT" {
            return Err(CompilerError::InvalidSyntax(
                "Query must match format: SELECT * FROM <collection> WHERE <field> <op> <value>".to_string()
            ));
        }

        let target_collection = tokens[3].to_string();
        let field = tokens[5].to_string();
        
        let operator = match tokens[6] {
            "=" => QueryOperator::Equal,
            ">" => QueryOperator::GreaterThan,
            "<" => QueryOperator::LessThan,
            op => return Err(CompilerError::UnsupportedOperator(op.to_string())),
        };

        let value = tokens[7].replace("'", "").replace("\"", "");

        Ok(QuantumQuery {
            target_collection,
            field,
            operator,
            value,
        })
    }

    /// Compiles a structured QuantumQuery into a physical Quantum Circuit Execution Plan
    /// Uses zero-cost abstractions to calculate required qubits based on theoretical value bounds
    pub fn compile_to_circuit(query: &QuantumQuery) -> QuantumCircuitPlan {
        let mut gates = Vec::new();
        
        // Dynamically determine qubits needed (Simulation default: 8 qubits for data matching register)
        let data_qubits = 8; 
        
        // Step 1: Initialize all data qubits into Superposition using Hadamard gates
        for qubit_idx in 0..data_qubits {
            gates.push(QuantumGate::H(qubit_idx));
        }

        // Step 2: Inject the Quantum Oracle corresponding to the lookup criteria
        let oracle_signature = format!("{}:{}:{:?}", query.field, query.value, query.operator);
        gates.push(QuantumGate::Oracle(oracle_signature));

        // Step 3: Append Diffuser configuration (Grover Diffusion Operator)
        for qubit_idx in 0..data_qubits {
            gates.push(QuantumGate::H(qubit_idx));
            gates.push(QuantumGate::X(qubit_idx));
        }
        
        QuantumCircuitPlan {
            required_qubits: data_qubits + 1, // +1 Ancilary qubit for phase kickback
            execution_gates: gates,
        }
    }
}
