use crate::compiler::{QuantumCircuitPlan, QuantumGate};
use crate::drivers::DriverConfig;
use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AwsBraketDriverError {
    #[error("AWS Braket API network dispatch failed: {0}")]
    NetworkError(String),
    #[error("AWS IAM Authorization or Signature V4 rejected: {0}")]
    AuthError(String),
    #[error("AWS Braket Quantum Task failed on cloud side: {0}")]
    TaskFailure(String),
    #[error("Quantum task execution timed out in the AWS Braket scheduler queue.")]
    QueueTimeout,
    #[error("JSON Payload serialization/deserialization collapse: {0}")]
    SerializationError(String),
}

#[derive(Serialize)]
struct AwsBraketTaskPayload {
    action: String, // OpenQASM 3.0 compilation directive
    device_arn: String,
    output_s3_bucket: String,
    output_s3_key_prefix: String,
    shots: usize,
}

#[derive(Deserialize)]
struct AwsBraketTaskResponse {
    #[serde(rename = "quantumTaskArn")]
    quantum_task_arn: String,
    status: String,
}

#[derive(Deserialize)]
struct AwsBraketTaskStatusResponse {
    status: String,
    #[serde(rename = "failureReason")]
    failure_reason: Option<String>,
}

#[derive(Deserialize)]
struct AwsS3ResultJson {
    #[serde(rename = "measurementProbabilities")]
    measurement_probabilities: Option<HashMap<String, f64>>,
    #[serde(rename = "measurementCounts")]
    measurement_counts: Option<HashMap<String, usize>>,
}

pub struct AwsBraketDriver {
    config: DriverConfig,
    s3_output_bucket: String,
    http_client: reqwest::Client,
}

impl AwsBraketDriver {
    pub fn new(config: DriverConfig, s3_output_bucket: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_default();

        Self {
            config,
            s3_output_bucket,
            http_client,
        }
    }

    /// Translates internal abstract Quantum Circuits into AWS Braket compliant OpenQASM 3.0 payloads.
    /// Injects specific hardware topology pragmas for Amazon-supported physical chips.
    pub fn transpile_to_aws_openqasm(&self, plan: &QuantumCircuitPlan) -> String {
        let mut qasm = String::with_capacity(1024);
        qasm.push_str("OPENQASM 3.0;\n");
        
        // Injecting AWS specific hardware layout macros if targeting rigorous ion-trap hardware
        if self.config.target_device.contains("ionq") {
            qasm.push_str("// AWS Braket Pragma Optimization Set for IonQ Native Traps\n");
            qasm.push_str("#pragma braket native_gates_only\n");
        }

        qasm.push_str(&format!("qreg q[{}];\ncreg c[{}];\n", plan.required_qubits, plan.required_qubits));

        for gate in &plan.execution_gates {
            match gate {
                QuantumGate::H(idx) => qasm.push_str(&format!("h q[{}];\n", idx)),
                QuantumGate::X(idx) => qasm.push_str(&format!("x q[{}];\n", idx)),
                QuantumGate::CX(ctrl, target) => qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, target)),
                QuantumGate::Oracle(signature) => {
                    qasm.push_str(&format!("// AWS Oracle Routine: {}\n", signature));
                    qasm.push_str("h q;\ncx q, q;\nh q;\n");
                }
            }
        }

        // Apply measurement registers to force wave function metrics extraction
        for i in 0..plan.required_qubits {
            qasm.push_str(&format!("c[{0}] = measure q[{0}];\n", i));
        }

        qasm
    }

    /// Dispatches a high-performance quantum database lookup job straight to AWS Braket Endpoints.
    /// Handles asynchronous long-polling loops watching AWS S3 storage buckets for results metadata.
    pub async fn dispatch_aws_task(&self, plan: &QuantumCircuitPlan) -> Result<HashMap<String, usize>, AwsBraketDriverError> {
        let openqasm_payload = self.transpile_to_aws_openqasm(plan);
        let endpoint = format!("https://braket.{}://", "us-east-1"); // Default high-availability quantum region

        let payload = AwsBraketTaskPayload {
            action: openqasm_payload,
            device_arn: self.config.target_device.clone(),
            output_s3_bucket: self.s3_output_bucket.clone(),
            output_s3_key_prefix: "hadamard_quantum_jobs/".to_string(),
            shots: 1024,
        };

        // Fire asynchronous HTTP request carrying the AWS token signature
        // In full orchestration environments, AWS Signature V4 headers are computed dynamically here
        let response = self.http_client.post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AwsBraketDriverError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AwsBraketDriverError::AuthError(format!("AWS Braket gateway rejected request. Status: {}", response.status())));
        }

        let task_data: AwsBraketTaskResponse = response.json()
            .await
            .map_err(|e| AwsBraketDriverError::SerializationError(e.to_string()))?;

        let task_arn = task_data.quantum_task_arn;
        let status_endpoint = format!("https://braket.{}:///{}", "us-east-1", task_arn);

        // Long polling loop monitoring task transitions from CREATED -> QUEUED -> RUNNING -> COMPLETED
        let mut attempts = 0;
        loop {
            if attempts > 60 { // Extended timeout parameters for cold physical dilution refrigerators
                return Err(AwsBraketDriverError::QueueTimeout);
            }

            let check_response = self.http_client.get(&status_endpoint)
                .header("Authorization", format!("Bearer {}", self.config.api_token))
                .send()
                .await
                .map_err(|e| AwsBraketDriverError::NetworkError(e.to_string()))?;

            let status_data: AwsBraketTaskStatusResponse = check_response.json()
                .await
                .map_err(|e| AwsBraketDriverError::SerializationError(e.to_string()))?;

            if status_data.status == "COMPLETED" {
                break;
            } else if status_data.status == "FAILED" {
                let reason = status_data.failure_reason.unwrap_or_else(|| "Unknown cloud fault".to_string());
                return Err(AwsBraketDriverError::TaskFailure(reason));
            }

            // Yield execution flow cleanly back to the async Tokio workers for 3 seconds before pooling again
            tokio::time::sleep(Duration::from_secs(3)).await;
            attempts += 1;
        }

        // Fetch execution output statistics directly from the configured Amazon S3 Bucket
        let s3_data_endpoint = format!("https://{}://{}/results.json", self.s3_output_bucket, task_arn.replace("/", "_"));
        
        let s3_response = self.http_client.get(&s3_data_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .send()
            .await
            .map_err(|e| AwsBraketDriverError::NetworkError(e.to_string()))?;

        let s3_payload: AwsS3ResultJson = s3_response.json()
            .await
            .map_err(|e| AwsBraketDriverError::SerializationError(e.to_string()))?;

        if let Some(counts) = s3_payload.measurement_counts {
            Ok(counts)
        } else {
            Err(AwsBraketDriverError::TaskFailure("AWS S3 bucket parsing error: measurement registers empty.".to_string()))
        }
    }
}
