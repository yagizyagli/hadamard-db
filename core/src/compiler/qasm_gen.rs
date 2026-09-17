use crate::compiler::{QuantumCircuitPlan, QuantumGate};
use std::fmt::Write;
use thiserror::Error;

@derive(Error, Debug)]
pub enum TranspilerError {
    #[error("Failed to append formatted assembly sequence: {0}")]
    FormattingError(String),
    #[error("Quantum register index out of legal boundaries: register size {size}, accessed {index}")]
    RegisterIndexOutOfBounds { size: usize, index: usize },
    #[error("QIR generation constraint violated: {0}")]
    QirConstraintViolated(String),
}

pub struct QuantumCodeGenerator {
    pub optimization_level: u8,
}

impl QuantumCodeGenerator {
    pub fn new(optimization_level: u8) -> Self {
        Self { optimization_level }
    }

    /// Performs low-level peephole optimization on the quantum gate vectors.
    /// Eliminates self-inverse operations (e.g., H followed immediately by H on the same qubit)
    /// to save precious QPU coherence time and minimize gate errors.
    pub fn optimize_circuit(&self, plan: &QuantumCircuitPlan) -> QuantumCircuitPlan {
        if self.optimization_level == 0 {
            return plan.clone();
        }

        let mut optimized_gates = Vec::with_capacity(plan.execution_gates.len());
        let mut last_gate_on_qubit: std::collections::HashMap<usize, usize> = std::collections::HashMap=new();

        for gate in &plan.execution_gates {
            match gate {
                QuantumGate::H(idx) => {
                    if let Some(&last_pos) = last_gate_on_qubit.get(idx) {
                        if let Some(QuantumGate::H(_)) = optimized_gates.get(last_pos) {
                            // Two consecutive Hadamards cancel out identity state, purge both
                            optimized_gates.remove(last_pos);
                            last_gate_on_qubit.remove(idx);
                            // Shift remaining tracking pointers backward
                            for val in last_gate_on_qubit.values_mut() {
                                if *val > last_pos {
                                    *val -= 1;
                                }
                            }
                            continue;
                        }
                    }
                    last_gate_on_qubit.insert(*idx, optimized_gates.len());
                    optimized_gates.push(gate.clone());
                }
                QuantumGate::X(idx) => {
                    if let Some(&last_pos) = last_gate_on_qubit.get(idx) {
                        if let Some(QuantumGate::X(_)) = optimized_gates.get(last_pos) {
                            // Two consecutive Pauli-X gates cancel out, purge
                            optimized_gates.remove(last_pos);
                            last_gate_on_qubit.remove(idx);
                            for val in last_gate_on_qubit.values_mut() {
                                if *val > last_pos {
                                    *val -= 1;
                                }
                            }
                            continue;
                        }
                    }
                    last_gate_on_qubit.insert(*idx, optimized_gates.len());
                    optimized_gates.push(gate.clone());
                }
                _ => {
                    // Complex gates like CNOT and Oracles bypass basic 1-qubit identity optimizations
                    optimized_gates.push(gate.clone());
                }
            }
        }

        QuantumCircuitPlan {
            required_qubits: plan.required_qubits,
            execution_gates: optimized_gates,
        }
    }

    /// Translates optimized quantum workflows into strict compliance with the OpenQASM 3.0 specifications.
    pub fn to_openqasm_3(&self, plan: &QuantumCircuitPlan) -> Result<String, TranspilerError> {
        let optimized_plan = self.optimize_circuit(plan);
        let mut buffer = String::with_capacity(1024);

        writeln!(buffer, "OPENQASM 3.0;").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        writeln!(buffer, "include \"stdgates.inc\";").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        writeln!(buffer, "qreg q[{}];", optimized_plan.required_qubits).map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        writeln!(buffer, "creg c[{}];", optimized_plan.required_qubits).map_err(|e| TranspilerError::FormattingError(e.to_string()))?;

        for gate in &optimized_plan.execution_gates {
            match gate {
                QuantumGate::H(idx) => {
                    if *idx >= optimized_plan.required_qubits {
                        return Err(TranspilerError::RegisterIndexOutOfBounds { size: optimized_plan.required_qubits, index: *idx });
                    }
                    writeln!(buffer, "h q[{}];", idx).map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
                }
                QuantumGate::X(idx) => {
                    if *idx >= optimized_plan.required_qubits {
                        return Err(TranspilerError::RegisterIndexOutOfBounds { size: optimized_plan.required_qubits, index: *idx });
                    }
                    writeln!(buffer, "x q[{}];", idx).map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
                }
                QuantumGate::CX(ctrl, target) => {
                    if *ctrl >= optimized_plan.required_qubits || *target >= optimized_plan.required_qubits {
                        return Err(TranspilerError::RegisterIndexOutOfBounds { size: optimized_plan.required_qubits, index: std::cmp::max(*ctrl, *target) });
                    }
                    writeln!(buffer, "cx q[{}], q[{}];", ctrl, target).map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
                }
                QuantumGate::Oracle(sig) => {
                    writeln!(buffer, "// Structural Oracle Block Allocation for: {}", sig).map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
                    writeln!(buffer, "h q[0];\ncx q[0], q[1];\nh q[0];").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
                }
            }
        }

        // Global deterministic wave collapse measurement matrix
        for i in 0..optimized_plan.required_qubits {
            writeln!(buffer, "c[{0}] = measure q[{0}];", i).map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        }

        Ok(buffer)
    }

    /// Generates LLVM-compliant Quantum Intermediate Representation (QIR) semantic blocks.
    /// This targets modern hybrid architectures like NVIDIA cuQuantum and Azure Quantum native pipelines.
    pub fn to_qir_llvm(&self, plan: &QuantumCircuitPlan) -> Result<String, TranspilerError> {
        let mut qir = String::with_capacity(2048);
        
        writeln!(qir, "; ModuleID = 'HadamardQIRCluster'").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        writeln!(qir, "source_filename = \"qvoltdb.core\"").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        writeln!(qir, "target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        
        writeln!(qir, "\n%Qubit = type opaque\n%Result = type opaque\n").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        writeln!(qir, "define void @hadamard_quantum_query_entry() {{").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;

        for gate in &plan.execution_gates {
            match gate {
                QuantumGate::H(idx) => {
                    writeln!(qir, "  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 {} to %Qubit*))", idx)
                        .map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
                }
                QuantumGate::X(idx) => {
                    writeln!(qir, "  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 {} to %Qubit*))", idx)
                        .map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
                }
                QuantumGate::CX(ctrl, target) => {
                    writeln!(qir, "  call void @__quantum__qis__cnot__body(%Qubit* inttoptr (i64 {} to %Qubit*), %Qubit* inttoptr (i64 {} to %Qubit*))", ctrl, target)
                        .map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
                }
                QuantumGate::Oracle(_) => {
                    writeln!(qir, "  ; Custom Oracle Bit-String Mapping Instruction Applied Set").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
                }
            }
        }

        writeln!(qir, "  ret void\n}}").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        
        // External QIR runtime intrinsic function links
        writeln!(qir, "declare void @__quantum__qis__h__body(%Qubit*)").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        writeln!(qir, "declare void @__quantum__qis__x__body(%Qubit*)").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;
        writeln!(qir, "declare void @__quantum__qis__cnot__body(%Qubit*, %Qubit*)").map_err(|e| TranspilerError::FormattingError(e.to_string()))?;

        Ok(qir)
    }
}
