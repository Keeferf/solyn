use tauri;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use std::time::Duration;
use crate::helpers::terminal_output_cleaner::{broadcast_terminal_line, parse_and_emit_terminal_output};
use crate::core::ollama::client::{is_ollama_installed, start_ollama, fetch_ollama_version};

pub async fn execute_ollama_installation(
    app_handle: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    platform: &str,
) -> Result<(), String> {
    let window_clone = window.clone();
    let shell = app_handle.shell();

    // The official installer writes to system locations, so it needs root on
    // Linux. A GUI process has no TTY, so `sudo` can't prompt — use the OS
    // elevation dialog instead, which shows a native password prompt and never
    // hands the password to this app.
    let (shell_cmd, shell_args): (String, Vec<String>) = match platform {
        "windows" => (
            "powershell".to_string(),
            vec!["-c".to_string(), "irm https://ollama.com/install.ps1 | iex".to_string()],
        ),
        "linux" => {
            let script_path = write_install_script()
                .await
                .map_err(|e| format!("Failed to prepare installer: {}", e))?;
            let script = script_path.to_string_lossy().to_string();

            if command_exists("pkexec") {
                // Linux desktop: polkit native dialog via the DE auth agent.
                ("pkexec".to_string(), vec!["/bin/sh".to_string(), script])
            } else if command_exists("run0") {
                // systemd >= 256 replacement for pkexec.
                ("run0".to_string(), vec!["/bin/sh".to_string(), script])
            } else {
                return Err(install_needs_elevation_error());
            }
        }
        _ => return Err("Unsupported platform".to_string()),
    };

    broadcast_terminal_line(&window_clone, &format!("Running installer for {}", platform), "info", false);

    let (mut rx, _child) = shell
        .command(&shell_cmd)
        .args(shell_args)
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    let mut exit_code: Option<i32> = None;
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
                exit_code = status.code;
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

    if exit_code != Some(0) {
        // pkexec exits 126 when the dialog is dismissed and 127 when it could
        // not authenticate (e.g. no polkit agent). Either way, surface the
        // manual command instead of pretending the install worked.
        if matches!(exit_code, Some(126) | Some(127)) {
            broadcast_terminal_line(&window_clone, "🔒 Could not show the system authentication dialog.", "error", false);
            broadcast_terminal_line(&window_clone, &format!("Run this in a terminal, then restart Solyn:\n  {}", manual_install_command()), "info", false);
            return Err("Ollama installation requires administrator privileges".to_string());
        }

        return Err(format!(
            "Ollama installation failed (exit code {}). Check the terminal log for details.",
            exit_code.map(|code| code.to_string()).unwrap_or_else(|| "unknown".to_string())
        ));
    }

    broadcast_terminal_line(window, "Verifying Ollama installation...", "info", false);
    
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
    
    // Final verification check
    broadcast_terminal_line(window, "Performing final verification check...", "info", false);
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    match is_ollama_installed().await {
        Ok(true) => {
            broadcast_terminal_line(window, "✓ Ollama verified and running!", "success", false);
            Ok(())
        }
        _ => {
            broadcast_terminal_line(window, "⚠️ Ollama installed but could not be verified.", "info", false);
            broadcast_terminal_line(window, "💡 Try starting Ollama manually, then refresh.", "info", false);
            Err("Ollama installation could not be verified".to_string())
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
    
    // Build the update command. On Linux the stock package-manager path is
    // unreliable: apt/pacman/snap only work for package-managed installs (the
    // official installer drops a binary in /usr/local/bin).
    //
    // Set when the update is handed off to a real terminal (WSL) instead of
    // being run directly from this TTY-less process.
    let mut launched_in_terminal = false;

    let (shell_cmd, shell_args): (String, Vec<String>) = match platform {
        "windows" => {
            // Windows: Try winget first (more reliable for updates), fallback to reinstall
            ("powershell".to_string(), vec![
                "-c".to_string(),
                "winget upgrade Ollama.Ollama --silent 2>$null; if ($LASTEXITCODE -ne 0) { irm https://ollama.com/install.ps1 | iex }".to_string(),
            ])
        }
        "linux" => {
            if is_wsl() && !sudo_nopasswd_available() {
                // WSL has no polkit agent, so pkexec can't prompt, and this GUI
                // process has no TTY for sudo. Hand the update to a real
                // terminal via Windows interop so sudo can ask for the password.
                let script_path = write_update_script()
                    .await
                    .map_err(|e| format!("Failed to prepare update script: {}", e))?;

                broadcast_terminal_line(&window_clone,
                    "🔒 Opening a terminal for the sudo password...",
                    "info", false
                );
                broadcast_terminal_line(&window_clone,
                    "Complete the password prompt in the window that opens; Solyn will continue automatically.",
                    "info", false
                );

                launch_wsl_update_terminal(&script_path)?;
                launched_in_terminal = true;
                (String::new(), Vec::new())
            } else {
                // Run the update script as the current user; it elevates with
                // `sudo -n` and exits 42 when that isn't possible.
                ("sh".to_string(), vec!["-c".to_string(), linux_update_script()])
            }
        }
        _ => return Err("Unsupported platform".to_string()),
    };

    if !launched_in_terminal {
        broadcast_terminal_line(&window_clone,
            &format!("📦 Running update for {}...", platform),
            "info", false
        );

        let (mut rx, _child) = shell
            .command(&shell_cmd)
            .args(shell_args)
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        // Process output and capture the real exit status so a failed update can't
        // be reported as success.
        let mut exit_code: Option<i32> = None;
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
                    exit_code = status.code;
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

        if exit_code != Some(0) {
            if exit_code == Some(42) {
                broadcast_terminal_line(&window_clone,
                    "🔒 Updating Ollama needs root, and no passwordless sudo is available.",
                    "error", false
                );
                broadcast_terminal_line(&window_clone,
                    "Run this in a terminal, then restart Solyn:\n  curl -fsSL https://ollama.com/install.sh | sudo sh && sudo systemctl restart ollama",
                    "info", false
                );
                return Err("Ollama update requires administrator privileges".to_string());
            }

            return Err(format!(
                "Ollama update failed (exit code {}). Check the update log for details.",
                exit_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ));
        }
    }

    // Verify the update was successful
    broadcast_terminal_line(window, "🔍 Verifying Ollama update...", "info", false);
    
    // Wait for the update to complete. The WSL terminal hand-off needs longer:
    // the user has to finish the sudo prompt and the installer has to download,
    // so allow up to ~3 minutes before giving up on a version change.
    let (max_attempts, unchanged_deadline) = if launched_in_terminal { (90, 60) } else { (25, 15) };
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
                        if version != current_version {
                            broadcast_terminal_line(window, 
                                &format!("✅ Ollama updated successfully to version {}", version), 
                                "success", false
                            );
                            return Ok(());
                        } else if attempts > unchanged_deadline {
                            // Script exited 0 but the running server still
                            // reports the old version: the binary was replaced
                            // but the process was never restarted (common on
                            // WSL/containers without systemd). Don't claim
                            // success.
                            broadcast_terminal_line(window,
                                &format!("⚠️ Ollama still reports version {} after the update.", version),
                                "error", false
                            );
                            broadcast_terminal_line(window,
                                "💡 Restart Ollama (or your WSL distro), then retry.",
                                "info", false
                            );
                            return Err(
                                "Ollama update did not change the running version. Restart Ollama, then retry.".to_string()
                            );
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
            broadcast_terminal_line(window, "⚠️ Update finished but Ollama is not responding.", "info", false);
            broadcast_terminal_line(window, "💡 Try starting Ollama manually, then refresh.", "info", false);
            Err("Ollama did not come back up after the update".to_string())
        }
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
        "linux" => "Follow the instructions at https://ollama.com/download/linux".to_string(),
        _ => "Visit https://ollama.com for installation instructions".to_string(),
    }
}

/// POSIX shell script that updates Ollama using whichever install method is
/// actually present, then restarts the service. Exits 42 when root is required
/// but `sudo` cannot run without a password prompt (a GUI process has no TTY).
fn linux_update_script() -> String {
    r#"
set -e
if [ "$(id -u)" -ne 0 ]; then
    if command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
        SUDO="sudo -n"
    else
        echo "SOLYN_SUDO_REQUIRED"
        exit 42
    fi
else
    SUDO=""
fi

if dpkg -l ollama 2>/dev/null | grep -q '^ii'; then
    $SUDO apt-get update
    $SUDO apt-get install --only-upgrade -y ollama
elif command -v pacman >/dev/null 2>&1 && pacman -Q ollama >/dev/null 2>&1; then
    $SUDO pacman -S --noconfirm ollama
elif command -v snap >/dev/null 2>&1 && snap list ollama >/dev/null 2>&1; then
    $SUDO snap refresh ollama
else
    tmp="$(mktemp)"
    curl -fsSL https://ollama.com/install.sh -o "$tmp"
    $SUDO sh "$tmp"
    rm -f "$tmp"
fi

if ! $SUDO systemctl restart ollama 2>/dev/null; then
    # No systemd (WSL, containers): stop the running server so the freshly
    # installed binary is what comes back up. Solyn restarts it afterwards.
    $SUDO pkill -x ollama 2>/dev/null || pkill -x ollama 2>/dev/null || true
fi
"#
    .to_string()
}

/// True when running inside WSL, where Windows binaries are reachable through
/// interop and there is normally no polkit agent for `pkexec` to talk to.
fn is_wsl() -> bool {
    std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::fs::read_to_string("/proc/version")
            .map(|v| v.to_ascii_lowercase().contains("microsoft"))
            .unwrap_or(false)
}

/// Whether `sudo` can run without prompting. When it can, the update stays in
/// the background; when it can't, WSL needs a real terminal for the password.
fn sudo_nopasswd_available() -> bool {
    std::process::Command::new("sudo")
        .args(["-n", "true"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Write the update script to a temp file so a terminal-run `sudo` can execute
/// it as root.
async fn write_update_script() -> Result<std::path::PathBuf, String> {
    let path = std::env::temp_dir().join(format!("solyn-ollama-update-{}.sh", std::process::id()));
    tokio::fs::write(&path, linux_update_script())
        .await
        .map_err(|e| format!("Failed to write update script: {}", e))?;
    Ok(path)
}

/// Open a Windows terminal (via WSL interop) running the update under `sudo`,
/// so the password prompt has a TTY. WSL has no polkit agent, so this is the
/// only way to elevate without pre-configuring passwordless sudo.
fn launch_wsl_update_terminal(script_path: &std::path::Path) -> Result<(), String> {
    use std::process::{Command, Stdio};

    let script = script_path.to_string_lossy().to_string();
    let distro = std::env::var("WSL_DISTRO_NAME").unwrap_or_default();

    // `wsl.exe -d <distro> -e sudo sh <script>` runs inside this distro with a
    // real TTY attached to the new window, so sudo can prompt.
    //
    // `wt.exe` cannot be launched from WSL (its execution alias doesn't work
    // there) and misparses the wsl flags, so go through cmd.exe's `start`, which
    // opens the user's default terminal.
    let mut args = vec!["/c".to_string(), "start".to_string(), "wsl.exe".to_string()];
    if !distro.is_empty() {
        args.push("-d".to_string());
        args.push(distro);
    }
    args.extend(["-e".to_string(), "sudo".to_string(), "sh".to_string(), script]);

    // The absolute path covers `appendWindowsPath=false`.
    let candidates: [(&str, Vec<String>); 2] = [
        ("cmd.exe", args.clone()),
        ("/mnt/c/Windows/System32/cmd.exe", args),
    ];

    for (program, args) in candidates {
        if Command::new(program)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
    }

    Err("Could not open a terminal to request administrator privileges. \
         Run the update in a terminal instead."
        .to_string())
}

/// Write the Ollama install script to a temp file. The elevated helper
/// (pkexec / run0) runs it as root; the script then downloads the
/// official installer and runs it, also as root, so the installer never needs to
/// call sudo itself.
async fn write_install_script() -> Result<std::path::PathBuf, String> {
    let path = std::env::temp_dir().join("solyn-ollama-install.sh");
    let script = r#"set -e
tmp="$(mktemp)"
curl -fsSL https://ollama.com/install.sh -o "$tmp"
/bin/sh "$tmp"
rm -f "$tmp"
"#;

    tokio::fs::write(&path, script)
        .await
        .map_err(|e| format!("Failed to write installer script: {}", e))?;

    Ok(path)
}

/// Check whether an executable exists on PATH (plus the usual system dirs, in
/// case the GUI process inherited a minimal PATH).
fn command_exists(name: &str) -> bool {
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();

    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    dirs.push(std::path::PathBuf::from("/usr/bin"));
    dirs.push(std::path::PathBuf::from("/bin"));

    dirs.iter().any(|dir| dir.join(name).exists())
}

/// The command a user can run themselves when no GUI elevation dialog exists.
fn manual_install_command() -> String {
    "curl -fsSL https://ollama.com/install.sh | sudo sh".to_string()
}

fn install_needs_elevation_error() -> String {
    format!(
        "Ollama installation needs administrator privileges, but no elevation dialog is available. \
         Run `{}` in a terminal, then restart Solyn.",
        manual_install_command()
    )
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