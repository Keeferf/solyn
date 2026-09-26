// src/core/installation/verifier.rs
use crate::core::ollama::client::{fetch_ollama_version, is_ollama_installed, is_ollama_running};

/// Quick verification - just check if Ollama is installed and running
pub async fn quick_verify_ollama() -> Result<bool, String> {
    match is_ollama_installed().await {
        Ok(true) => match is_ollama_running().await {
            Ok(true) => Ok(true),
            _ => Ok(false),
        },
        _ => Ok(false),
    }
}

/// Check if Ollama is ready to use (installed and running)
pub async fn is_ollama_ready() -> Result<bool, String> {
    match quick_verify_ollama().await {
        Ok(true) => Ok(true),
        _ => Ok(false),
    }
}

/// Verify with detailed status information
pub async fn verify_ollama_with_details() -> Result<OllamaVerificationStatus, String> {
    let is_installed = is_ollama_installed().await.unwrap_or(false);
    let is_running = if is_installed {
        is_ollama_running().await.unwrap_or(false)
    } else {
        false
    };

    let version = if is_running {
        fetch_ollama_version().await.ok()
    } else {
        None
    };

    Ok(OllamaVerificationStatus {
        is_installed,
        is_running,
        version,
        platform: crate::core::platform::detector::detect_operating_system(),
    })
}

/// Detailed verification status
#[derive(Debug, Clone, serde::Serialize)]
pub struct OllamaVerificationStatus {
    pub is_installed: bool,
    pub is_running: bool,
    pub version: Option<String>,
    pub platform: String,
}
