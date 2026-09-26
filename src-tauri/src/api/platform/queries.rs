use super::contracts::{PlatformInfo, SystemResources};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn get_platform_info() -> String {
    crate::core::platform::detector::detect_operating_system()
}

#[tauri::command]
pub async fn get_platform_info_detailed(_app_handle: AppHandle) -> Result<PlatformInfo, String> {
    let os = crate::core::platform::detector::detect_operating_system();

    let arch = std::env::consts::ARCH.to_string();

    let version = if cfg!(target_os = "windows") {
        std::env::var("OS").ok()
    } else if cfg!(target_os = "linux") {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("PRETTY_NAME="))
                    .and_then(|line| line.split('=').nth(1))
                    .map(|s| s.trim_matches('"').to_string())
            })
    } else {
        None
    };

    Ok(PlatformInfo {
        os,
        arch: Some(arch),
        version,
    })
}

#[tauri::command]
pub async fn get_system_resources() -> Result<SystemResources, String> {
    Ok(SystemResources {
        memory_total: None,
        memory_available: None,
        cpu_cores: Some(num_cpus::get()),
    })
}

#[tauri::command]
pub async fn get_app_version(_app_handle: AppHandle) -> Result<String, String> {
    let version = env!("CARGO_PKG_VERSION");
    Ok(version.to_string())
}

#[tauri::command]
pub async fn get_app_data_path(app_handle: AppHandle) -> Result<String, String> {
    let path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    Ok(path.to_str().unwrap_or("").to_string())
}
