// src/core/ollama/models.rs
use reqwest;
use serde_json::json;
use std::time::Duration;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaModelList {
    pub models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaRunningModel {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OllamaPsList {
    pub models: Vec<OllamaRunningModel>,
}

/// Client for Ollama model management
pub struct OllamaModelClient {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaModelClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    /// Create a new model using the Modelfile path.
    ///
    /// Uses the `ollama` CLI on every platform. The CLI reads the GGUF locally
    /// and streams it to the server, so it works even when the server runs as a
    /// different user (the Linux systemd service runs as `ollama` and cannot
    /// read the app's data dir). The HTTP `/api/create` endpoint instead makes
    /// the server open the path itself, which fails on Linux and for relative
    /// `FROM ./file` paths in general.
    pub async fn create_model(&self, model_name: &str, modelfile_path: &PathBuf) -> Result<String, String> {
        let ollama_path = find_ollama_executable()?;

        // `FROM ./file` in the Modelfile is resolved relative to the working
        // directory, so run the CLI from the directory that holds the GGUF.
        let working_dir = modelfile_path.parent().unwrap_or_else(|| std::path::Path::new("."));

        let mut command = tokio::process::Command::new(&ollama_path);
        command
            .args(["create", model_name, "-f"])
            .arg(modelfile_path)
            .current_dir(working_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let output = command
            .output()
            .await
            .map_err(|e| format!("Failed to execute ollama create: {}", e))?;

        if output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Check if there was any error output
            if stderr.contains("error") || stderr.contains("Error") {
                return Err(format!("Ollama error: {}", stderr));
            }

            Ok(format!("Model '{}' created successfully", model_name))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to create model: {}", stderr))
        }
    }

    /// List available models (returns just names)
    pub async fn list_models(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self.client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to list models: {}", e))?;

        if response.status().is_success() {
            let data: OllamaModelList = response.json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            
            let models: Vec<String> = data.models
                .into_iter()
                .map(|m| m.name)
                .collect();
            
            Ok(models)
        } else {
            Err("Failed to list models".to_string())
        }
    }

    /// Get all models with full details
    pub async fn get_all_models(&self) -> Result<Vec<OllamaModel>, String> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self.client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to list models: {}", e))?;

        if response.status().is_success() {
            let data: OllamaModelList = response.json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            
            Ok(data.models)
        } else {
            Err("Failed to list models".to_string())
        }
    }

    /// Delete a model
    pub async fn delete_model(&self, model_name: &str) -> Result<(), String> {
        let url = format!("{}/api/delete", self.base_url);
        
        let payload = json!({
            "name": model_name,
        });

        let response = self.client
            .delete(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to delete model: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err("Failed to delete model".to_string())
        }
    }

    /// Get model details by name
    pub async fn get_model_details(&self, model_name: &str) -> Result<OllamaModel, String> {
        let all_models = self.get_all_models().await?;
        
        for model in all_models {
            if model.name == model_name {
                return Ok(model);
            }
        }
        
        Err(format!("Model '{}' not found", model_name))
    }

    /// Check if a model exists
    pub async fn model_exists(&self, model_name: &str) -> Result<bool, String> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m == model_name))
    }

    /// Check if a model is currently loaded in memory (warm).
    /// Uses `/api/ps`, which lists running models.
    pub async fn is_model_loaded(&self, model_name: &str) -> Result<bool, String> {
        let url = format!("{}/api/ps", self.base_url);

        let response = self.client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to list running models: {}", e))?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let data: OllamaPsList = response.json()
            .await
            .map_err(|e| format!("Failed to parse /api/ps: {}", e))?;

        let base = |n: &str| n.split(':').next().unwrap_or(n).to_string();
        Ok(data.models.iter().any(|m| {
            m.name == model_name || base(&m.name) == base(model_name)
        }))
    }

    /// Check Ollama health/version
    pub async fn check_health(&self) -> Result<bool, String> {
        let response = self.client
            .get(&format!("{}/api/version", self.base_url))
            .timeout(Duration::from_secs(2))
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
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .map_err(|e| format!("Failed to get version: {}", e))?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            
            if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                Ok(version.to_string())
            } else {
                Err("Version not found in response".to_string())
            }
        } else {
            Err("Failed to get version".to_string())
        }
    }
}

impl Default for OllamaModelClient {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to find the ollama executable on Linux.
// GUI apps can launch with a minimal PATH, so check common install locations
// first, then fall back to relying on PATH.
#[cfg(not(target_os = "windows"))]
fn find_ollama_executable() -> Result<String, String> {
    use std::path::Path;

    let common_paths = [
        "/usr/local/bin/ollama",
        "/usr/bin/ollama",
        "/snap/bin/ollama",
        "/var/lib/flatpak/exports/bin/ollama",
    ];

    for path in common_paths.iter() {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }

    Ok("ollama".to_string())
}

// Helper function to find ollama executable on Windows
#[cfg(target_os = "windows")]
fn find_ollama_executable() -> Result<String, String> {
    use std::env;
    use std::path::Path;
    
    let common_paths = [
        r"C:\Program Files\Ollama\ollama.exe",
        r"C:\Program Files (x86)\Ollama\ollama.exe",
        r"%LOCALAPPDATA%\Programs\Ollama\ollama.exe",
        r"%USERPROFILE%\AppData\Local\Programs\Ollama\ollama.exe",
    ];
    
    for path in common_paths.iter() {
        let expanded_path = if path.starts_with('%') {
            let path_str = path.to_string();
            if path_str.contains("%LOCALAPPDATA%") {
                if let Ok(localappdata) = env::var("LOCALAPPDATA") {
                    path_str.replace("%LOCALAPPDATA%", &localappdata)
                } else {
                    continue;
                }
            } else if path_str.contains("%USERPROFILE%") {
                if let Ok(userprofile) = env::var("USERPROFILE") {
                    path_str.replace("%USERPROFILE%", &userprofile)
                } else {
                    continue;
                }
            } else {
                path_str
            }
        } else {
            path.to_string()
        };
        
        if Path::new(&expanded_path).exists() {
            return Ok(expanded_path);
        }
    }
    
    // Try using 'where' command
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    
    let output = std::process::Command::new("where")
        .arg("ollama")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Failed to find ollama: {}", e))?;
    
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .ok_or_else(|| "No ollama found".to_string())?
            .trim()
            .to_string();
        
        if !path.is_empty() {
            return Ok(path);
        }
    }
    
    Err("Could not find ollama executable. Please ensure Ollama is installed.".to_string())
}