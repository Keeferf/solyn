// src/core/huggingface/installed.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tokio::fs;

use super::utils::{extract_parameter_count, extract_quantization, ollama_model_name};
use crate::core::ollama::models::OllamaModelClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModelFile {
    pub filename: String,
    pub size: u64,
    pub path: String,
    pub parameter_count: Option<String>,
    pub quantization: Option<String>,
    pub has_modelfile: bool,
    pub modelfile_name: Option<String>, // Track which Modelfile belongs to this file
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModel {
    pub model_id: String,
    pub author: String,
    pub name: String,
    pub files: Vec<InstalledModelFile>,
    pub total_size: u64,
    pub downloaded_at: String,
    pub has_modelfile: bool,
}

/// Get all installed models from the app data directory
pub async fn get_installed_models(app_handle: &AppHandle) -> Result<Vec<InstalledModel>, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let models_dir = app_dir.join("models");

    if !models_dir.exists() {
        return Ok(Vec::new());
    }

    let mut installed_models = Vec::new();
    let mut entries = fs::read_dir(&models_dir)
        .await
        .map_err(|e| format!("Failed to read models directory: {}", e))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
    {
        let path = entry.path();
        if path.is_dir() {
            if let Some(model) = scan_model_directory(&path).await {
                installed_models.push(model);
            }
        }
    }

    // Sort by downloaded_at (newest first)
    installed_models.sort_by(|a, b| b.downloaded_at.cmp(&a.downloaded_at));

    Ok(installed_models)
}

/// Scan a single model directory for GGUF files
async fn scan_model_directory(dir_path: &PathBuf) -> Option<InstalledModel> {
    let dir_name = dir_path.file_name()?.to_str()?;

    // Parse model_id from directory name (author_modelname format)
    let parts: Vec<&str> = dir_name.split('_').collect();
    let author = parts.first().unwrap_or(&"").to_string();
    let name = parts.get(1).unwrap_or(&"").to_string();
    let model_id = format!("{}/{}", author, name);

    let mut files = Vec::new();
    let mut total_size = 0;
    // `read_dir` order is arbitrary, so check for the Modelfile up front
    // instead of relying on encountering it before a .gguf file.
    let modelfile_exists = dir_path.join("Modelfile").exists();

    let metadata = fs::metadata(dir_path).await.ok()?;
    let downloaded_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| {
            let secs = d.as_secs();
            let days = secs / 86400;
            if days > 0 {
                format!("{} day{} ago", days, if days > 1 { "s" } else { "" })
            } else {
                let hours = secs / 3600;
                if hours > 0 {
                    format!("{} hour{} ago", hours, if hours > 1 { "s" } else { "" })
                } else {
                    let mins = secs / 60;
                    if mins > 0 {
                        format!("{} minute{} ago", mins, if mins > 1 { "s" } else { "" })
                    } else {
                        "Just now".to_string()
                    }
                }
            }
        })
        .unwrap_or_else(|| "Unknown".to_string());

    // Read directory contents
    let mut entries = fs::read_dir(dir_path).await.ok()?;
    while let Some(entry) = entries.next_entry().await.ok()? {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name()?.to_str()?.to_string();

            // Modelfiles are not model files; presence is tracked via
            // `modelfile_exists` above.
            if filename == "Modelfile" {
                continue;
            }

            if filename.ends_with(".gguf") {
                let size = fs::metadata(&path).await.ok().map(|m| m.len()).unwrap_or(0);
                total_size += size;

                let parameter_count = extract_parameter_count(&filename);
                let quantization = extract_quantization(&filename);

                let modelfile_name = if modelfile_exists {
                    Some("Modelfile".to_string())
                } else {
                    None
                };

                let file_info = InstalledModelFile {
                    filename,
                    size,
                    path: path.to_str()?.to_string(),
                    parameter_count,
                    quantization,
                    has_modelfile: modelfile_name.is_some(),
                    modelfile_name,
                };
                files.push(file_info);
            }
        }
    }

    if files.is_empty() {
        return None;
    }

    Some(InstalledModel {
        model_id,
        author,
        name,
        files,
        total_size,
        downloaded_at,
        has_modelfile: modelfile_exists,
    })
}

/// Ollama model names that correspond to a downloaded GGUF file.
/// Must match the naming used in `generate_modelfile` / `get_chat_models`.
fn ollama_model_names(model_id: &str, filename: &str) -> Vec<String> {
    let name = ollama_model_name(model_id, extract_quantization(filename).as_deref());
    vec![name.clone(), format!("{}:latest", name)]
}

/// Unregister the Ollama model(s) for a GGUF file. Best-effort: the model may
/// already be gone or Ollama may be offline, neither of which should block
/// deleting the local file.
async fn remove_ollama_model(model_id: &str, filename: &str) {
    let client = OllamaModelClient::new();
    for name in ollama_model_names(model_id, filename) {
        let _ = client.delete_model(&name).await;
    }
}

/// Delete an installed model and its files
pub async fn delete_installed_model(app_handle: &AppHandle, model_id: &str) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let model_folder_name = model_id.replace("/", "_");
    let model_dir = app_dir.join("models").join(&model_folder_name);

    if !model_dir.exists() {
        return Err(format!("Model directory not found: {}", model_id));
    }

    // Collect the GGUF filenames before removing the directory so the
    // matching Ollama models can be unregistered too.
    let mut gguf_files = Vec::new();
    let mut entries = fs::read_dir(&model_dir)
        .await
        .map_err(|e| format!("Failed to read model directory: {}", e))?;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
    {
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(".gguf") {
                gguf_files.push(name.to_string());
            }
        }
    }

    // Remove the entire directory
    fs::remove_dir_all(&model_dir)
        .await
        .map_err(|e| format!("Failed to delete model: {}", e))?;

    for filename in &gguf_files {
        remove_ollama_model(model_id, filename).await;
    }

    Ok(())
}

/// Delete a single file from an installed model
pub async fn delete_model_file(
    app_handle: &AppHandle,
    model_id: &str,
    filename: &str,
) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let model_folder_name = model_id.replace("/", "_");
    let file_path = app_dir
        .join("models")
        .join(&model_folder_name)
        .join(filename);

    if !file_path.exists() {
        return Err(format!("File not found: {}", filename));
    }

    // Don't allow deleting Modelfiles directly
    if filename == "Modelfile" {
        return Err("Cannot delete Modelfile directly. Delete the GGUF file instead.".to_string());
    }

    // Remove the file
    fs::remove_file(&file_path)
        .await
        .map_err(|e| format!("Failed to delete file: {}", e))?;

    remove_ollama_model(model_id, filename).await;

    // Also delete associated Modelfile if it exists (always "Modelfile" now)
    let modelfile_name = "Modelfile".to_string();
    let modelfile_path = file_path.parent().unwrap().join(&modelfile_name);
    if modelfile_path.exists() {
        let _ = fs::remove_file(&modelfile_path).await;
    }

    // Check if directory is empty after deletion
    let model_dir = file_path.parent().unwrap();
    let mut entries = fs::read_dir(model_dir)
        .await
        .map_err(|e| format!("Failed to read model directory: {}", e))?;
    let has_any_files = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
        .is_some();

    // If no files left, delete the entire model directory
    if !has_any_files {
        fs::remove_dir_all(model_dir)
            .await
            .map_err(|e| format!("Failed to remove empty model directory: {}", e))?;
    }

    Ok(())
}

/// Delete all files with a specific quantization
pub async fn delete_model_quantization(
    app_handle: &AppHandle,
    model_id: &str,
    quantization: &str,
) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let model_folder_name = model_id.replace("/", "_");
    let model_dir = app_dir.join("models").join(&model_folder_name);

    if !model_dir.exists() {
        return Err(format!("Model directory not found: {}", model_id));
    }

    let mut deleted_count = 0;
    let mut entries = fs::read_dir(&model_dir)
        .await
        .map_err(|e| format!("Failed to read model directory: {}", e))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
    {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip Modelfiles directly
            if filename == "Modelfile" {
                continue;
            }

            // Check if this file has the target quantization
            if let Some(file_quant) = extract_quantization(filename) {
                if file_quant == quantization {
                    // Delete the file
                    fs::remove_file(&path)
                        .await
                        .map_err(|e| format!("Failed to delete file {}: {}", filename, e))?;

                    remove_ollama_model(model_id, filename).await;

                    // Delete associated Modelfile (always "Modelfile" now)
                    let modelfile_name = "Modelfile".to_string();
                    let modelfile_path = model_dir.join(&modelfile_name);
                    if modelfile_path.exists() {
                        let _ = fs::remove_file(&modelfile_path).await;
                    }

                    deleted_count += 1;
                }
            }
        }
    }

    if deleted_count == 0 {
        return Err(format!(
            "No files found with quantization: {}",
            quantization
        ));
    }

    // Check if directory is empty after deletion
    let mut entries = fs::read_dir(&model_dir)
        .await
        .map_err(|e| format!("Failed to read model directory: {}", e))?;
    let has_any_files = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
        .is_some();

    // If no files left, delete the entire model directory
    if !has_any_files {
        fs::remove_dir_all(&model_dir)
            .await
            .map_err(|e| format!("Failed to remove empty model directory: {}", e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ollama_model_names, scan_model_directory};

    #[test]
    fn ollama_names_include_quantization_and_latest() {
        let names = ollama_model_names("ornith-ai/Ornith-1.5-9B-GGUF", "Ornith-1.5-9B-Q8_0.gguf");
        assert_eq!(
            names,
            vec![
                "ornith-ai_Ornith-1.5-9B-GGUF_Q8_0".to_string(),
                "ornith-ai_Ornith-1.5-9B-GGUF_Q8_0:latest".to_string(),
            ]
        );
    }

    #[test]
    fn ollama_names_fall_back_to_base_without_quantization() {
        let names = ollama_model_names("author/model", "model.gguf");
        assert_eq!(
            names,
            vec![
                "author_model".to_string(),
                "author_model:latest".to_string()
            ]
        );
    }

    #[test]
    fn ollama_names_extract_quantization_from_hyphenated_filename() {
        // The old `split('_')` heuristic returned no quantization here.
        let names = ollama_model_names(
            "meta-llama/Llama-3.1-8B-Instruct",
            "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf",
        );
        assert_eq!(
            names,
            vec![
                "meta-llama_Llama-3.1-8B-Instruct_Q4_K_M".to_string(),
                "meta-llama_Llama-3.1-8B-Instruct_Q4_K_M:latest".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn scan_model_directory_reads_gguf_and_modelfile() {
        let dir = tempfile::tempdir().unwrap();
        let model_dir = dir.path().join("author_modelname");
        std::fs::create_dir(&model_dir).unwrap();
        std::fs::write(model_dir.join("model-Q4_K_M.gguf"), vec![0u8; 128]).unwrap();
        std::fs::write(model_dir.join("Modelfile"), "FROM ./model-Q4_K_M.gguf").unwrap();
        std::fs::write(model_dir.join("README.md"), "docs").unwrap();

        let model = scan_model_directory(&model_dir).await.unwrap();

        assert_eq!(model.model_id, "author/modelname");
        assert_eq!(model.author, "author");
        assert_eq!(model.name, "modelname");
        assert_eq!(model.files.len(), 1);
        assert_eq!(model.total_size, 128);
        assert_eq!(model.files[0].filename, "model-Q4_K_M.gguf");
        assert_eq!(model.files[0].quantization.as_deref(), Some("Q4_K_M"));
        assert!(model.files[0].has_modelfile);
        assert_eq!(model.files[0].modelfile_name.as_deref(), Some("Modelfile"));
        // Must not depend on the arbitrary read_dir order (Modelfile was
        // written after the .gguf here).
        assert!(model.has_modelfile);
    }

    #[tokio::test]
    async fn scan_model_directory_has_no_modelfile_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let model_dir = dir.path().join("author_modelname");
        std::fs::create_dir(&model_dir).unwrap();
        std::fs::write(model_dir.join("model.gguf"), b"x").unwrap();

        let model = scan_model_directory(&model_dir).await.unwrap();
        assert!(!model.has_modelfile);
        assert!(!model.files[0].has_modelfile);
        assert!(model.files[0].modelfile_name.is_none());
    }

    #[tokio::test]
    async fn scan_model_directory_returns_none_without_gguf() {
        let dir = tempfile::tempdir().unwrap();
        let model_dir = dir.path().join("author_modelname");
        std::fs::create_dir(&model_dir).unwrap();
        std::fs::write(model_dir.join("README.md"), "docs").unwrap();

        assert!(scan_model_directory(&model_dir).await.is_none());
    }
}
