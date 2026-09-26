// src/core/huggingface/download/paths.rs

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// All paths related to a model download
pub struct ModelPaths {
    pub model_dir: PathBuf,
    pub file_path: PathBuf,
    pub part_path: PathBuf,
}

impl ModelPaths {
    pub fn new(app_handle: &AppHandle, model_id: &str, filename: &str) -> Result<Self, String> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;

        let models_dir = app_dir.join("models");
        let model_folder_name = model_id.replace("/", "_");
        let model_dir = models_dir.join(&model_folder_name);
        let file_path = model_dir.join(filename);
        let part_path = model_dir.join(format!("{}.part", filename));

        Ok(Self {
            model_dir,
            file_path,
            part_path,
        })
    }

    /// Get chunk file path for a specific chunk index
    pub fn chunk_path(&self, filename: &str, chunk_index: usize) -> PathBuf {
        self.model_dir
            .join(format!("{}.part.{}", filename, chunk_index))
    }
}

/// Clean up chunk files
pub async fn cleanup_chunks(paths: &ModelPaths, filename: &str, num_chunks: usize) {
    for i in 0..num_chunks {
        let _ = tokio::fs::remove_file(paths.chunk_path(filename, i)).await;
    }
}
