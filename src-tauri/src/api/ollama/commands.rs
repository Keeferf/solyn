use tauri;
use tauri::Manager;

use super::contracts::OllamaStatus;
use crate::core::installation::executor::{execute_ollama_installation, execute_ollama_update}; // Add execute_ollama_update
use crate::core::ollama::client::*;
use crate::core::platform::detector::detect_operating_system;
use crate::data::download_state::DownloadStatus;
use crate::events::ollama_status_monitor::OllamaStatusMonitor;
use crate::events::progress_broadcaster::broadcast_download_progress;

#[tauri::command]
pub async fn start_ollama_service(app_handle: tauri::AppHandle) -> Result<String, String> {
    start_ollama(&app_handle).await
}

#[tauri::command]
pub async fn download_ollama(app_handle: tauri::AppHandle) -> Result<String, String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let platform = detect_operating_system();

    broadcast_download_progress(
        &window,
        DownloadStatus::Downloading,
        0,
        "Starting download...".to_string(),
        None,
    );

    let download_result = execute_ollama_installation(&app_handle, &window, &platform).await;

    if let Err(e) = download_result {
        broadcast_download_progress(
            &window,
            DownloadStatus::Error,
            0,
            format!("Download failed: {}", e),
            None,
        );
        return Err(e);
    }

    // Trigger a status update after successful installation
    let monitor = OllamaStatusMonitor::new();
    monitor.trigger_status_check(&app_handle).await;

    broadcast_download_progress(
        &window,
        DownloadStatus::Complete,
        100,
        "Ollama installed successfully!".to_string(),
        None,
    );

    Ok("Ollama installed successfully".to_string())
}

#[tauri::command]
pub async fn update_ollama(app_handle: tauri::AppHandle) -> Result<String, String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let platform = detect_operating_system();

    broadcast_download_progress(
        &window,
        DownloadStatus::Downloading,
        0,
        "Starting Ollama update...".to_string(),
        None,
    );

    // Use the DEDICATED update function instead of installation
    let update_result = execute_ollama_update(&app_handle, &window, &platform).await;

    if let Err(e) = update_result {
        broadcast_download_progress(
            &window,
            DownloadStatus::Error,
            0,
            format!("Update failed: {}", e),
            None,
        );
        return Err(e);
    }

    // Wait a moment for the update to fully settle
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Trigger a status update after successful installation
    let monitor = OllamaStatusMonitor::new();
    monitor.trigger_status_check(&app_handle).await;

    broadcast_download_progress(
        &window,
        DownloadStatus::Complete,
        100,
        "Ollama updated successfully!".to_string(),
        None,
    );

    Ok("Ollama updated successfully".to_string())
}

#[tauri::command]
pub async fn refresh_ollama_status(app_handle: tauri::AppHandle) -> Result<OllamaStatus, String> {
    let monitor = OllamaStatusMonitor::new();
    monitor.trigger_status_check(&app_handle).await;

    // Also return the current status immediately
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
