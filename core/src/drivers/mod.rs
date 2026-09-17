pub mod ibm;
pub mod aws;
pub mod azure;
pub mod gcp; 

#[derive(Debug, Clone)]
pub enum QuantumBackendProvider {
    IbmQuantum,
    AwsBraket,
    AzureQuantum,
    GcpQuantumEngine, // Expanded for Google Cloud topology
    LocalSimulator,
}

#[derive(Debug, Clone)]
pub struct DriverConfig {
    pub provider: QuantumBackendProvider,
    pub api_token: String,
    pub target_device: String,
    pub timeout_seconds: u64,
}
