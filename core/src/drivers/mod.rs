pub mod ibm;

#[derive(Debug, Clone)]
pub enum QuantumBackendProvider {
    IbmQuantum,
    AwsBraket,
    AzureQuantum,
    LocalSimulator,
}

#[derive(Debug, Clone)]
pub struct DriverConfig {
    pub provider: QuantumBackendProvider,
    pub api_token: String,
    pub target_device: String, // e.g., "ibm_brisbane" or "simulator"
    pub timeout_seconds: u64,
}
