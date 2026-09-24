use reqwest;
use serde_json;
use std::time::Duration;
use crate::data::download_state::InstallationInformation;
use crate::helpers::platform_detector::detect_operating_system;
use tauri::AppHandle;

pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    /// Check if Ollama is running
    pub async fn check_health(&self) -> Result<bool, String> {
        let response = self.client
            .get(&format!("{}/api/version", self.base_url))
            .timeout(Duration::from_millis(1500))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => Ok(true),
            _ => Ok(false),
        }
    }

    /// Get Ollama version
    pub async fn get_version(&self) -> Result<String, String> {
        let response = self.client
            .get(&format!("{}/api/version", self.base_url))
            .timeout(Duration::from_millis(1500))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if response.status().is_success() {
            let version_info: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse version: {}", e))?;
            Ok(version_info["version"]
                .as_str()
                .unwrap_or("unknown")
                .to_string())
        } else {
            Err("Ollama is not running".to_string())
        }
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Standalone Functions =====

/// Check if Ollama is installed on the system (OPTIMIZED)
pub async fn is_ollama_installed() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        use std::env;
        
        // Method 1: Check common installation paths first (no timeouts, instant)
        let common_paths = [
            r"C:\Program Files\Ollama\ollama.exe",
            r"C:\Program Files (x86)\Ollama\ollama.exe",
        ];
        
        for path in common_paths.iter() {
            if Path::new(path).exists() {
                return Ok(true);
            }
        }
        
        // Method 2: Check %LOCALAPPDATA%
        if let Ok(localappdata) = env::var("LOCALAPPDATA") {
            let path = format!(r"{}\Programs\Ollama\ollama.exe", localappdata);
            if Path::new(&path).exists() {
                return Ok(true);
            }
        }
        
        // Method 3: Check %USERPROFILE%
        if let Ok(userprofile) = env::var("USERPROFILE") {
            let path = format!(r"{}\AppData\Local\Programs\Ollama\ollama.exe", userprofile);
            if Path::new(&path).exists() {
                return Ok(true);
            }
        }
        
        // Method 4: Try using where command (no timeout = instant on success, normal on failure)
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let output = std::process::Command::new("where")
            .arg("ollama")
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .stdin(std::process::Stdio::null())
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                return Ok(true);
            }
        }
        
        // Skip registry checks—they're 200ms each with minimal ROI
        Ok(false)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        use std::path::Path;

        // GUI apps often inherit a minimal PATH, so check common install
        // locations before falling back to `which`.
        let common_paths = [
            "/usr/local/bin/ollama",
            "/usr/bin/ollama",
            "/snap/bin/ollama",
            "/var/lib/flatpak/exports/bin/ollama",
        ];
        if common_paths.iter().any(|p| Path::new(p).exists()) {
            return Ok(true);
        }

        match std::process::Command::new("which")
            .arg("ollama")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output() 
        {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }
}

/// Check if Ollama is running (server is responsive) - OPTIMIZED
pub async fn is_ollama_running() -> Result<bool, String> {
    match fetch_ollama_version().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Fetch the Ollama version - OPTIMIZED with shorter timeout
pub async fn fetch_ollama_version() -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:11434/api/version")
        .timeout(Duration::from_millis(1500))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

    if response.status().is_success() {
        let version_info: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse version: {}", e))?;
        Ok(version_info["version"]
            .as_str()
            .unwrap_or("unknown")
            .to_string())
    } else {
        Err("Ollama is not running".to_string())
    }
}

/// Start Ollama service with adaptive polling (OPTIMIZED - no hard sleep)
pub async fn start_ollama(_app_handle: &AppHandle) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        
        let ollama_paths = [
            r"C:\Program Files\Ollama\ollama.exe",
            r"C:\Program Files (x86)\Ollama\ollama.exe",
        ];
        
        let mut started = false;
        for path in ollama_paths.iter() {
            if std::path::Path::new(path).exists() {
                let child = std::process::Command::new(path)
                    .arg("serve")
                    .creation_flags(CREATE_NEW_PROCESS_GROUP)
                    .spawn();
                    
                if child.is_ok() {
                    started = true;
                    break;
                }
            }
        }
        
        if !started {
            // Fallback to PATH lookup
            let child = std::process::Command::new("ollama")
                .arg("serve")
                .creation_flags(CREATE_NEW_PROCESS_GROUP)
                .spawn();
                
            if child.is_err() {
                return Err("Failed to start Ollama".to_string());
            }
        }
        
        // ADAPTIVE POLLING: Poll every 250ms for up to 3 seconds (12 attempts)
        // instead of hard sleep(3s)
        for _attempt in 1..=12 {
            tokio::time::sleep(Duration::from_millis(250)).await;
            
            if is_ollama_running().await? {
                return Ok("Ollama started successfully".to_string());
            }
        }
        
        Err("Ollama failed to start after 3 seconds".to_string())
    }
    
    #[cfg(target_os = "linux")]
    {
        // The official installer registers a *system* service named `ollama`;
        // some setups use a user service. Try both, then fall back to launching
        // the server directly (containers without systemd).
        for args in [
            vec!["start", "ollama.service"],
            vec!["--user", "start", "ollama.service"],
        ] {
            let _ = std::process::Command::new("systemctl")
                .args(&args)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            for _ in 0..4 {
                tokio::time::sleep(Duration::from_millis(500)).await;
                if is_ollama_running().await? {
                    return Ok("Ollama started successfully".to_string());
                }
            }
        }

        let _ = std::process::Command::new("ollama")
            .arg("serve")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        for _ in 0..8 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if is_ollama_running().await? {
                return Ok("Ollama started successfully".to_string());
            }
        }

        Err("Ollama is starting but not ready yet".to_string())
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Unsupported platform for starting Ollama".to_string())
    }
}

pub async fn get_installation_instructions() -> Result<InstallationInformation, String> {
    let platform = detect_operating_system();
    
    let (command, estimated_time) = match platform.as_str() {
        "windows" => (
            "irm https://ollama.com/install.ps1 | iex".to_string(),
            "~5 minutes".to_string(),
        ),
        "linux" => (
            "curl -fsSL https://ollama.com/install.sh | sh".to_string(),
            "~5 minutes".to_string(),
        ),
        _ => return Err("Unsupported platform".to_string()),
    };

    Ok(InstallationInformation {
        platform,
        command,
        estimated_time,
    })
}