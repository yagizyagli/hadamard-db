use hadamard_core::compiler::{QuantumCircuitPlan, QuantumGate};
use hadamard_core::compiler::qasm_gen::QuantumCodeGenerator;

#[test]
fn test_quantum_circuit_peephole_gate_cancellation_optimization() {
    let generator = QuantumCodeGenerator::new(3); // Enable max optimization pipelines
    
    // Construct a theoretical circuit containing redundant consecutive self-inverse gates (Hadamard -> Hadamard)
    let raw_gates = vec![
        QuantumGate::H(0),
        QuantumGate::H(0), // Redundant, should be eliminated entirely by the optimizer
        QuantumGate::X(1),
        QuantumGate::H(2),
    ];

    let experimental_plan = QuantumCircuitPlan {
        required_qubits: 3,
        execution_gates: raw_gates,
    };

    let optimized_plan = generator.optimize_circuit(&experimental_plan);
    
    // Verification: Qubit 0 gates should be completely purged, total gates reduced to 2 (X and H)
    assert_eq!(optimized_plan.execution_gates.len(), 2, "Gate optimizer failed to purge inverse identity pairs.");
    
    if let QuantumGate::X(idx) = optimized_plan.execution_gates[0] {
        assert_eq!(idx, 1);
    } else {
        panic!("Circuit gate processing order corrupted during optimization sweep.");
    }
}

#[test]
fn test_openqasm_3_transpilation_syntax_compliance() {
    let generator = QuantumCodeGenerator::new(1);
    let plan = QuantumCircuitPlan {
        required_qubits: 2,
        execution_gates: vec![
            QuantumGate::H(0),
            QuantumGate::CX(0, 1),
        ],
    };

    let qasm_result = generator.to_openqasm_3(&plan);
    assert!(qasm_result.is_ok(), "Transpiler crashed while processing raw compliant gate vectors.");
    
    let qasm_string = qasm_result.unwrap();
    
    // Strict compliance checks against standard OpenQASM 3.0 regex tokens
    assert!(qasm_string.contains("OPENQASM 3.0;"), "Missing OpenQASM specification header.");
    assert!(qasm_string.contains("qreg q[2];"), "Quantum register allocation token missing or incorrect layout sizing.");
    assert!(qasm_string.contains("h q[0];"), "Hadamard transformation instruction formatting violation.");
    assert!(qasm_string.contains("cx q[0], q[1];"), "Controlled-NOT matrix instruction syntax error.");
    assert!(qasm_string.contains("measure"), "Wavefunction state measurement final flags are missing.");
}

#[test]
fn test_quantum_register_overflow_security_boundaries() {
    let generator = QuantumCodeGenerator::new(0);
    
    // Adversarial vector: Accessing qubit register index 5 on a circuit that only requested 4 qubits
    let broken_plan = QuantumCircuitPlan {
        required_qubits: 4,
        execution_gates: vec![
            QuantumGate::H(5), // Register Overflow! Should trigger a safe error bubble up
        ],
    };

    let transpilation_result = generator.to_openqasm_3(&broken_plan);
    assert!(transpilation_result.is_less(), "Transpiler failed to catch register index overflow vulnerabilities.");
}
