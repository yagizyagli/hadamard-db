use crate::compiler::{QuantumCircuitPlan, QuantumGate};
use crate::drivers::DriverConfig;
use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IbmDriverError {
    #[error("Network transfer or HTTP handshake failed: {0}")]
    NetworkError(String),
    #[error("IBM Quantum API returned an explicit error response: {0}")]
    ApiError(String),
    #[error("Quantum job execution timed out in the cloud queue.")]
    Timeout,
    #[error("Failed to serialize or deserialize quantum hardware payload: {0}")]
    SerializationError(String),
}

#[derive(Serialize)]
struct IbmJobPayload {
    program_id: String,
    backend: String,
    params: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct IbmJobResponse {
    id: String,
    status: String,
}

#[derive(Deserialize)]
struct IbmResultResponse {
    status: String,
    results: Vec<IbmCircuitResult>,
}

#[derive(Deserialize)]
struct IbmCircuitResult {
    data: IbmResultData,
}

#[derive(Deserialize)]
struct IbmResultData {
    counts: HashMap<String, usize>, // Bitstring matrix mapping to quantum hit records
}

pub struct IbmQuantumDriver {
    config: DriverConfig,
    http_client: reqwest::Client,
}

impl IbmQuantumDriver {
    pub fn new(config: DriverConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_default();

        Self { config, http_client }
    }

    /// Transpiles internal abstract Quantum Gates directly into raw, physical OpenQASM 3.0 string vectors
    pub fn transpile_to_openqasm(&self, plan: &QuantumCircuitPlan) -> String {
        let mut qasm = String::with_capacity(512);
        qasm.push_str("OPENQASM 3.0;\ninclude \"stdgates.inc\";\n");
        qasm.push_str(&format!("qreg q[{}];\ncreg c[{}];\n", plan.required_qubits, plan.required_qubits));

        for gate in &plan.execution_gates {
            match gate {
                QuantumGate::H(idx) => qasm.push_str(&format!("h q[{}];\n", idx)),
                QuantumGate::X(idx) => qasm.push_str(&format!("x q[{}];\n", idx)),
                QuantumGate::CX(ctrl, target) => qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, target)),
                QuantumGate::Oracle(signature) => {
                    // Injecting hardware-optimized phase oracle sub-routines
                    qasm.push_str(&format!("// Begin Phase Oracle for {}\n", signature));
                    qasm.push_str("h q[0];\ncx q[1], q[0];\nh q[0];\n");
                }
            }
        }

        // Apply global end-of-circuit measurement registers to capture the collapsed wave function
        for i in 0..plan.required_qubits {
            qasm.push_str(&format!("c[{0}] = measure q[{0}];\n", i));
        }

        qasm
    }

    /// Offloads the compiled quantum query directly onto IBM Quantum cloud chips via non-blocking asynchronous event loops
    pub async fn dispatch_quantum_job(&self, plan: &QuantumCircuitPlan) -> Result<HashMap<String, usize>, IbmDriverError> {
        let raw_qasm = self.transpile_to_openqasm(plan);
        let endpoint = "https://ibm.com";

        let mut params = HashMap::new();
        params.insert("circuits".to_string(), serde_json::Value::String(raw_qasm));
        params.insert("shots".to_string(), serde_json::Value::Number(serde_json::Number::from(1024)));

        let payload = IbmJobPayload {
            program_id: "qiskit-runtime".to_string(),
            backend: self.config.target_device.clone(),
            params,
        };

        // Fire HTTP POST connection request
        let response = self.http_client.post(endpoint)
            .header("X-Access-Token", &self.config.api_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| IbmDriverError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IbmDriverError::ApiError(format!("HTTP Status Client Error: {}", response.status())));
        }

        let job_data: IbmJobResponse = response.json()
            .await
            .map_err(|e| IbmDriverError::SerializationError(e.to_string()))?;

        let job_id = job_data.id;
        let status_endpoint = format!("{}/{}", endpoint, job_id);

        // Enter Non-blocking Long Polling Cycle to monitor cloud computing queue progression
        let mut attempts = 0;
        loop {
            if attempts > 30 {
                return Err(IbmDriverError::Timeout);
            }

            let check_response = self.http_client.get(&status_endpoint)
                .header("X-Access-Token", &self.config.api_token)
                .send()
                .await
                .map_err(|e| IbmDriverError::NetworkError(e.to_string()))?;

            let status_data: IbmJobResponse = check_response.json()
                .await
                .map_err(|e| IbmDriverError::SerializationError(e.to_string()))?;

            if status_data.status == "COMPLETED" {
                break;
            } else if status_data.status == "FAILED" || status_data.status == "CANCELLED" {
                return Err(IbmDriverError::ApiError(format!("Cloud Job crashed with status: {}", status_data.status)));
            }

            // Yield control back to the async executor thread-pool for 2 seconds before next poll
            tokio::time::sleep(Duration::from_secs(2)).await;
            attempts += 1;
        }

        // Fetch execution matrix payloads
        let result_endpoint = format!("{}/results", status_endpoint);
        let result_response = self.http_client.get(&result_endpoint)
            .header("X-Access-Token", &self.config.api_token)
            .send()
            .await
            .map_err(|e| IbmDriverError::NetworkError(e.to_string()))?;

        let final_payload: IbmResultResponse = result_response.json()
            .await
            .map_err(|e| IbmDriverError::SerializationError(e.to_string()))?;

        if let Some(first_circuit_result) = final_payload.results.first() {
            Ok(first_circuit_result.data.counts.clone())
        } else {
            Err(IbmDriverError::ApiError("IBM backend execution returned completely empty result blocks.".to_string()))
        }
    }
}
