// src/core/huggingface/download/paths.rs

use std::path::{Path, PathBuf};
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

        Ok(Self::in_dir(&app_dir, model_id, filename))
    }

    /// Path layout under an app data directory. Split out of `new` so tests
    /// don't need a tauri `AppHandle`.
    pub fn in_dir(app_dir: &Path, model_id: &str, filename: &str) -> Self {
        let models_dir = app_dir.join("models");
        let model_folder_name = model_id.replace("/", "_");
        let model_dir = models_dir.join(&model_folder_name);
        let file_path = model_dir.join(filename);
        let part_path = model_dir.join(format!("{}.part", filename));

        Self {
            model_dir,
            file_path,
            part_path,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn data_dir() -> PathBuf {
        PathBuf::from("data")
    }

    #[test]
    fn folds_slashes_into_underscores() {
        let paths = ModelPaths::in_dir(&data_dir(), "meta-llama/Llama-3", "m.gguf");
        let model_dir = data_dir().join("models").join("meta-llama_Llama-3");

        assert_eq!(paths.model_dir, model_dir);
        assert_eq!(paths.file_path, model_dir.join("m.gguf"));
        assert_eq!(paths.part_path, model_dir.join("m.gguf.part"));
    }

    #[test]
    fn chunk_path_appends_index() {
        let paths = ModelPaths::in_dir(&data_dir(), "a/b", "m.gguf");
        assert_eq!(
            paths.chunk_path("m.gguf", 3),
            data_dir().join("models").join("a_b").join("m.gguf.part.3")
        );
    }
}