use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub memory_total: Option<u64>,
    pub memory_available: Option<u64>,
    pub cpu_cores: Option<usize>,
}
