use tauri;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use std::time::Duration;
use crate::helpers::terminal_output_cleaner::{broadcast_terminal_line, parse_and_emit_terminal_output};
use crate::core::ollama::client::{is_ollama_installed, is_ollama_running, start_ollama, fetch_ollama_version};

pub async fn execute_ollama_installation(
    app_handle: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    platform: &str,
) -> Result<(), String> {
    let window_clone = window.clone();
    let shell = app_handle.shell();
    
    let (shell_cmd, script_cmd) = match platform {
        "windows" => ("powershell", "irm https://ollama.com/install.ps1 | iex"),
        "macos" => ("sh", "curl -fsSL https://ollama.com/install.sh | sh"),
        "linux" => ("sh", "curl -fsSL https://ollama.com/install.sh | sh"),
        _ => return Err("Unsupported platform".to_string()),
    };

    broadcast_terminal_line(&window_clone, &format!("Running installer for {}", platform), "info", false);

    let (mut rx, _child) = shell
        .command(shell_cmd)
        .args(&["-c", script_cmd])
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    while let Some(event) = rx.recv().await {
        match event {
            tauri_plugin_shell::process::CommandEvent::Stdout(data) => {
                if let Ok(text) = String::from_utf8(data) {
                    parse_and_emit_terminal_output(&window_clone, &text, "stdout");
                }
            }
            tauri_plugin_shell::process::CommandEvent::Stderr(data) => {
                if let Ok(text) = String::from_utf8(data) {
                    parse_and_emit_terminal_output(&window_clone, &text, "stderr");
                }
            }
            tauri_plugin_shell::process::CommandEvent::Terminated(status) => {
                let msg = if status.code == Some(0) {
                    "Installation script completed"
                } else {
                    "Process terminated with error"
                };
                broadcast_terminal_line(&window_clone, msg, "info", false);
            }
            _ => {}
        }
    }

    broadcast_terminal_line(window, "Verifying Ollama installation...", "info", false);
    
    // EXACT same verification logic as original installation_executor.rs
    let max_attempts = 15;
    let mut attempts = 0;
    
    while attempts < max_attempts {
        tokio::time::sleep(Duration::from_secs(2)).await;
        attempts += 1;
        
        match is_ollama_installed().await {
            Ok(true) => {
                broadcast_terminal_line(window, "Ollama verified and running", "success", false);
                return Ok(());
            }
            Ok(false) => {
                if attempts < max_attempts && attempts % 3 == 0 {
                    broadcast_terminal_line(window, &format!("Waiting for Ollama to start... (attempt {}/{})", attempts, max_attempts), "info", false);
                }
            }
            Err(_e) => {
                if attempts < max_attempts && attempts % 3 == 0 {
                    broadcast_terminal_line(window, &format!("Checking Ollama status... (attempt {}/{})", attempts, max_attempts), "info", false);
                }
            }
        }
    }
    
    // Perform final verification check (EXACT same as original)
    broadcast_terminal_line(window, "Performing final verification check...", "info", false);
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    match is_ollama_installed().await {
        Ok(true) => {
            broadcast_terminal_line(window, "✓ Ollama verified and running!", "success", false);
            Ok(())
        }
        _ => {
            broadcast_terminal_line(window, "⚠️ Ollama installation completed but verification timed out.", "info", false);
            broadcast_terminal_line(window, "The installation should be complete. You can try refreshing the page.", "info", false);
            broadcast_terminal_line(window, "💡 If you see this message repeatedly, Ollama may need to be started manually.", "info", false);
            Ok(())
        }
    }
}

/// Execute Ollama UPDATE process (NEW)
/// This forces a reinstallation/update even if Ollama is already installed
pub async fn execute_ollama_update(
    app_handle: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    platform: &str,
) -> Result<(), String> {
    let window_clone = window.clone();
    let shell = app_handle.shell();
    
    // Get current version for logging
    let current_version = match fetch_ollama_version().await {
        Ok(v) => v,
        Err(_) => "unknown".to_string(),
    };
    
    broadcast_terminal_line(&window_clone, 
        &format!("🔄 Updating Ollama from version {}...", current_version), 
        "info", false
    );
    
    // Use different update methods per platform
    let (shell_cmd, script_cmd) = match platform {
        "windows" => {
            // Windows: Try winget first (more reliable for updates), fallback to reinstall
            ("powershell", "winget upgrade Ollama.Ollama --silent 2>$null; if ($LASTEXITCODE -ne 0) { irm https://ollama.com/install.ps1 | iex }")
        }
        "macos" => {
            // macOS: Try brew if available, otherwise re-run installer
            ("sh", "if command -v brew &> /dev/null; then brew upgrade ollama; else curl -fsSL https://ollama.com/install.sh | sh; fi")
        }
        "linux" => {
            // Linux: Try multiple methods
            ("sh", r#"
                if command -v apt &> /dev/null; then
                    sudo apt update && sudo apt install --only-upgrade ollama -y
                elif command -v pacman &> /dev/null; then
                    sudo pacman -Syu ollama --noconfirm
                elif command -v snap &> /dev/null; then
                    sudo snap refresh ollama
                else
                    curl -fsSL https://ollama.com/install.sh | sh
                fi
            "#)
        }
        _ => return Err("Unsupported platform".to_string()),
    };

    broadcast_terminal_line(&window_clone, 
        &format!("📦 Running update for {}...", platform), 
        "info", false
    );

    let (mut rx, _child) = shell
        .command(shell_cmd)
        .args(&["-c", script_cmd])
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    // Process output
    while let Some(event) = rx.recv().await {
        match event {
            tauri_plugin_shell::process::CommandEvent::Stdout(data) => {
                if let Ok(text) = String::from_utf8(data) {
                    parse_and_emit_terminal_output(&window_clone, &text, "stdout");
                }
            }
            tauri_plugin_shell::process::CommandEvent::Stderr(data) => {
                if let Ok(text) = String::from_utf8(data) {
                    parse_and_emit_terminal_output(&window_clone, &text, "stderr");
                }
            }
            tauri_plugin_shell::process::CommandEvent::Terminated(status) => {
                let msg = if status.code == Some(0) {
                    "✅ Update script completed"
                } else {
                    "⚠️ Update process terminated with error"
                };
                broadcast_terminal_line(&window_clone, msg, "info", false);
            }
            _ => {}
        }
    }

    // Verify the update was successful
    broadcast_terminal_line(window, "🔍 Verifying Ollama update...", "info", false);
    
    // Wait for the update to complete with extended timeout
    let max_attempts = 25;
    let mut attempts = 0;
    let mut ollama_started = false;
    
    while attempts < max_attempts {
        tokio::time::sleep(Duration::from_secs(2)).await;
        attempts += 1;
        
        // Check if Ollama is installed
        match is_ollama_installed().await {
            Ok(true) => {
                // Check if it's running and get version
                match fetch_ollama_version().await {
                    Ok(version) => {
                        // Version should be different (or at least we assume it updated)
                        if version != current_version || attempts > 15 {
                            broadcast_terminal_line(window, 
                                &format!("✅ Ollama updated successfully to version {}", version), 
                                "success", false
                            );
                            return Ok(());
                        } else if attempts % 3 == 0 {
                            broadcast_terminal_line(window, 
                                &format!("⏳ Waiting for version change... (attempt {}/{})", attempts, max_attempts), 
                                "info", false
                            );
                        }
                    }
                    Err(_) => {
                        // Try to start Ollama if it's not running
                        if !ollama_started && attempts % 3 == 0 {
                            broadcast_terminal_line(window, "🚀 Attempting to start Ollama...", "info", false);
                            let _ = start_ollama(app_handle).await;
                            ollama_started = true;
                        }
                        
                        if attempts < max_attempts && attempts % 3 == 0 {
                            broadcast_terminal_line(window, 
                                &format!("⏳ Waiting for Ollama to start... (attempt {}/{})", attempts, max_attempts), 
                                "info", false
                            );
                        }
                    }
                }
            }
            Ok(false) => {
                if attempts < max_attempts && attempts % 3 == 0 {
                    broadcast_terminal_line(window, 
                        &format!("⏳ Waiting for Ollama installation... (attempt {}/{})", attempts, max_attempts), 
                        "info", false
                    );
                }
            }
            Err(_e) => {
                if attempts < max_attempts && attempts % 3 == 0 {
                    broadcast_terminal_line(window, 
                        &format!("⏳ Checking Ollama status... (attempt {}/{})", attempts, max_attempts), 
                        "info", false
                    );
                }
            }
        }
    }
    
    // Final verification with one last attempt
    broadcast_terminal_line(window, "🔍 Performing final verification...", "info", false);
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    // Try one more time to start Ollama
    if !ollama_started {
        broadcast_terminal_line(window, "🚀 One final attempt to start Ollama...", "info", false);
        let _ = start_ollama(app_handle).await;
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    
    match fetch_ollama_version().await {
        Ok(version) => {
            broadcast_terminal_line(window, 
                &format!("✅ Ollama updated to version {}", version), 
                "success", false
            );
            Ok(())
        }
        Err(_) => {
            broadcast_terminal_line(window, "⚠️ Update may have completed but verification failed.", "info", false);
            broadcast_terminal_line(window, "💡 Try restarting Ollama manually or refreshing the page.", "info", false);
            Ok(())
        }
    }
}

/// Execute Ollama installation with automatic startup attempt
/// This version adds auto-start behavior that the original had
pub async fn execute_ollama_installation_with_auto_start(
    app_handle: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    platform: &str,
) -> Result<(), String> {
    let window_clone = window.clone();
    let shell = app_handle.shell();
    
    let (shell_cmd, script_cmd) = match platform {
        "windows" => ("powershell", "irm https://ollama.com/install.ps1 | iex"),
        "macos" => ("sh", "curl -fsSL https://ollama.com/install.sh | sh"),
        "linux" => ("sh", "curl -fsSL https://ollama.com/install.sh | sh"),
        _ => return Err("Unsupported platform".to_string()),
    };

    broadcast_terminal_line(&window_clone, &format!("Running installer for {}", platform), "info", false);

    let (mut rx, _child) = shell
        .command(shell_cmd)
        .args(&["-c", script_cmd])
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    while let Some(event) = rx.recv().await {
        match event {
            tauri_plugin_shell::process::CommandEvent::Stdout(data) => {
                if let Ok(text) = String::from_utf8(data) {
                    parse_and_emit_terminal_output(&window_clone, &text, "stdout");
                }
            }
            tauri_plugin_shell::process::CommandEvent::Stderr(data) => {
                if let Ok(text) = String::from_utf8(data) {
                    parse_and_emit_terminal_output(&window_clone, &text, "stderr");
                }
            }
            tauri_plugin_shell::process::CommandEvent::Terminated(status) => {
                let msg = if status.code == Some(0) {
                    "Installation script completed"
                } else {
                    "Process terminated with error"
                };
                broadcast_terminal_line(&window_clone, msg, "info", false);
            }
            _ => {}
        }
    }

    broadcast_terminal_line(window, "Verifying Ollama installation...", "info", false);
    
    // Enhanced verification with auto-start
    let max_attempts = 15;
    let mut attempts = 0;
    let mut started_ollama = false;
    
    while attempts < max_attempts {
        tokio::time::sleep(Duration::from_secs(2)).await;
        attempts += 1;
        
        match is_ollama_installed().await {
            Ok(true) => {
                // Check if running
                match is_ollama_running().await {
                    Ok(true) => {
                        broadcast_terminal_line(window, "Ollama verified and running", "success", false);
                        return Ok(());
                    }
                    Ok(false) => {
                        // Installed but not running - try to start it
                        if !started_ollama {
                            broadcast_terminal_line(window, "Ollama is installed but not running. Attempting to start...", "info", false);
                            let _ = start_ollama(app_handle).await;
                            started_ollama = true;
                        }
                        
                        if attempts < max_attempts && attempts % 3 == 0 {
                            broadcast_terminal_line(window, &format!("Waiting for Ollama to start... (attempt {}/{})", attempts, max_attempts), "info", false);
                        }
                    }
                    Err(_e) => {
                        if attempts < max_attempts && attempts % 3 == 0 {
                            broadcast_terminal_line(window, &format!("Checking Ollama status... (attempt {}/{})", attempts, max_attempts), "info", false);
                        }
                    }
                }
            }
            Ok(false) => {
                if attempts < max_attempts && attempts % 3 == 0 {
                    broadcast_terminal_line(window, &format!("Waiting for Ollama to install... (attempt {}/{})", attempts, max_attempts), "info", false);
                }
            }
            Err(_e) => {
                if attempts < max_attempts && attempts % 3 == 0 {
                    broadcast_terminal_line(window, &format!("Checking Ollama status... (attempt {}/{})", attempts, max_attempts), "info", false);
                }
            }
        }
    }
    
    // Final verification with one last auto-start attempt
    broadcast_terminal_line(window, "Performing final verification check...", "info", false);
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    match is_ollama_installed().await {
        Ok(true) => {
            match is_ollama_running().await {
                Ok(true) => {
                    broadcast_terminal_line(window, "✓ Ollama verified and running!", "success", false);
                    Ok(())
                }
                _ => {
                    // One last attempt to start
                    broadcast_terminal_line(window, "Attempting to start Ollama one final time...", "info", false);
                    let _ = start_ollama(app_handle).await;
                    
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    
                    match is_ollama_running().await {
                        Ok(true) => {
                            broadcast_terminal_line(window, "✓ Ollama started successfully!", "success", false);
                            Ok(())
                        }
                        _ => {
                            broadcast_terminal_line(window, "⚠️ Ollama installation completed but verification timed out.", "info", false);
                            broadcast_terminal_line(window, "The installation should be complete. You can try refreshing the page.", "info", false);
                            broadcast_terminal_line(window, "💡 If you see this message repeatedly, Ollama may need to be started manually.", "info", false);
                            Ok(())
                        }
                    }
                }
            }
        }
        _ => {
            broadcast_terminal_line(window, "⚠️ Ollama installation completed but verification timed out.", "info", false);
            broadcast_terminal_line(window, "The installation should be complete. You can try refreshing the page.", "info", false);
            broadcast_terminal_line(window, "💡 If you see this message repeatedly, Ollama may need to be started manually.", "info", false);
            Ok(())
        }
    }
}

/// Execute Ollama installation with custom verification attempts
pub async fn execute_ollama_installation_with_retry(
    app_handle: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    platform: &str,
    max_attempts: u32,
) -> Result<(), String> {
    let window_clone = window.clone();
    let shell = app_handle.shell();
    
    let (shell_cmd, script_cmd) = match platform {
        "windows" => ("powershell", "irm https://ollama.com/install.ps1 | iex"),
        "macos" => ("sh", "curl -fsSL https://ollama.com/install.sh | sh"),
        "linux" => ("sh", "curl -fsSL https://ollama.com/install.sh | sh"),
        _ => return Err("Unsupported platform".to_string()),
    };

    broadcast_terminal_line(&window_clone, &format!("Running installer for {}", platform), "info", false);

    let (mut rx, _child) = shell
        .command(shell_cmd)
        .args(&["-c", script_cmd])
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    while let Some(event) = rx.recv().await {
        match event {
            tauri_plugin_shell::process::CommandEvent::Stdout(data) => {
                if let Ok(text) = String::from_utf8(data) {
                    parse_and_emit_terminal_output(&window_clone, &text, "stdout");
                }
            }
            tauri_plugin_shell::process::CommandEvent::Stderr(data) => {
                if let Ok(text) = String::from_utf8(data) {
                    parse_and_emit_terminal_output(&window_clone, &text, "stderr");
                }
            }
            tauri_plugin_shell::process::CommandEvent::Terminated(status) => {
                let msg = if status.code == Some(0) {
                    "Installation script completed"
                } else {
                    "Process terminated with error"
                };
                broadcast_terminal_line(&window_clone, msg, "info", false);
            }
            _ => {}
        }
    }

    broadcast_terminal_line(window, "Verifying Ollama installation...", "info", false);
    
    let mut attempts = 0;
    let mut started_ollama = false;
    
    while attempts < max_attempts {
        tokio::time::sleep(Duration::from_secs(2)).await;
        attempts += 1;
        
        match is_ollama_installed().await {
            Ok(true) => {
                match is_ollama_running().await {
                    Ok(true) => {
                        broadcast_terminal_line(window, "Ollama verified and running", "success", false);
                        return Ok(());
                    }
                    Ok(false) => {
                        if !started_ollama {
                            broadcast_terminal_line(window, "Attempting to start Ollama...", "info", false);
                            let _ = start_ollama(app_handle).await;
                            started_ollama = true;
                        }
                        if attempts < max_attempts && attempts % 3 == 0 {
                            broadcast_terminal_line(window, &format!("Waiting for Ollama to start... (attempt {}/{})", attempts, max_attempts), "info", false);
                        }
                    }
                    Err(_e) => {
                        if attempts < max_attempts && attempts % 3 == 0 {
                            broadcast_terminal_line(window, &format!("Checking Ollama status... (attempt {}/{})", attempts, max_attempts), "info", false);
                        }
                    }
                }
            }
            Ok(false) => {
                if attempts < max_attempts && attempts % 3 == 0 {
                    broadcast_terminal_line(window, &format!("Waiting for Ollama to install... (attempt {}/{})", attempts, max_attempts), "info", false);
                }
            }
            Err(_e) => {
                if attempts < max_attempts && attempts % 3 == 0 {
                    broadcast_terminal_line(window, &format!("Checking Ollama status... (attempt {}/{})", attempts, max_attempts), "info", false);
                }
            }
        }
    }
    
    if !started_ollama {
        broadcast_terminal_line(window, "Performing final verification check...", "info", false);
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        match is_ollama_installed().await {
            Ok(true) => {
                match is_ollama_running().await {
                    Ok(true) => {
                        broadcast_terminal_line(window, "✓ Ollama verified and running!", "success", false);
                        Ok(())
                    }
                    _ => {
                        broadcast_terminal_line(window, "Attempting to start Ollama one final time...", "info", false);
                        let _ = start_ollama(app_handle).await;
                        
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        
                        match is_ollama_running().await {
                            Ok(true) => {
                                broadcast_terminal_line(window, "✓ Ollama started successfully!", "success", false);
                                Ok(())
                            }
                            _ => {
                                broadcast_terminal_line(window, "⚠️ Ollama installation completed but verification timed out.", "info", false);
                                broadcast_terminal_line(window, "💡 Ollama may need to be started manually.", "info", false);
                                Ok(())
                            }
                        }
                    }
                }
            }
            _ => {
                broadcast_terminal_line(window, "⚠️ Ollama installation completed but verification timed out.", "info", false);
                broadcast_terminal_line(window, "💡 Ollama may need to be started manually.", "info", false);
                Ok(())
            }
        }
    } else {
        Ok(())
    }
}

/// Save installation log
pub async fn save_installation_log(app_handle: &tauri::AppHandle, log_content: &str) -> Result<std::path::PathBuf, String> {
    use tokio::fs;
    
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    let log_dir = app_dir.join("logs");
    let log_path = log_dir.join("ollama_install.log");
    
    if let Some(parent) = log_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create log directory: {}", e))?;
        }
    }
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let formatted_log = format!("[{}] Installation Log\n{}\n{}\n", 
        timestamp, 
        "=".repeat(50),
        log_content
    );
    
    fs::write(&log_path, formatted_log)
        .await
        .map_err(|e| format!("Failed to write log file: {}", e))?;
    
    Ok(log_path)
}

/// Get installation recommendation based on platform
pub fn get_installation_recommendation(platform: &str) -> String {
    match platform {
        "windows" => "Download the Ollama installer from https://ollama.com/download/windows".to_string(),
        "macos" => "Download the Ollama app from https://ollama.com/download/mac".to_string(),
        "linux" => "Follow the instructions at https://ollama.com/download/linux".to_string(),
        _ => "Visit https://ollama.com for installation instructions".to_string(),
    }
}

/// Check if Ollama is installed via package manager (Linux only)
#[cfg(target_os = "linux")]
pub async fn check_package_manager_ollama() -> Result<bool, String> {
    let apt_check = std::process::Command::new("dpkg")
        .args(&["-l", "ollama"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();
    
    if let Ok(output) = apt_check {
        if output.status.success() {
            return Ok(true);
        }
    }
    
    let pacman_check = std::process::Command::new("pacman")
        .args(&["-Q", "ollama"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();
    
    if let Ok(output) = pacman_check {
        if output.status.success() {
            return Ok(true);
        }
    }
    
    Ok(false)
}

#[cfg(not(target_os = "linux"))]
pub async fn check_package_manager_ollama() -> Result<bool, String> {
    Ok(false)
}