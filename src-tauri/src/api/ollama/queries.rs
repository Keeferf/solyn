use super::contracts::OllamaStatus;
use crate::core::ollama::client::*;
use crate::data::download_state::InstallationInformation;

#[tauri::command]
pub async fn check_ollama_installed() -> Result<bool, String> {
    is_ollama_installed().await
}

#[tauri::command]
pub async fn check_ollama_running() -> Result<bool, String> {
    is_ollama_running().await
}

#[tauri::command]
pub async fn get_ollama_status() -> Result<OllamaStatus, String> {
    let installed = is_ollama_installed().await?;
    let (running, version) = if installed {
        match fetch_ollama_version().await {
            Ok(v) => (true, Some(v)),
            Err(_) => (false, None),
        }
    } else {
        (false, None)
    };

    Ok(OllamaStatus {
        installed,
        running,
        version,
    })
}

#[tauri::command]
pub async fn get_ollama_version() -> Result<String, String> {
    fetch_ollama_version().await
}

#[tauri::command]
pub async fn get_install_info() -> Result<InstallationInformation, String> {
    get_installation_instructions().await
}
