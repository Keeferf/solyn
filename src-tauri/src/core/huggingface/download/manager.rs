// src/core/huggingface/download/manager.rs

use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::AppHandle;  // Removed Manager since it's unused
use tokio::fs;

use super::chunking_logic::*;
use super::paths::*;
use super::progress_reporting::*;

use crate::core::huggingface::cache::{generate_download_id, insert_cancellation_token, remove_cancellation_token};
use crate::core::huggingface::modelfile::{write_modelfile, ModelFileConfig, write_metadata};
use crate::core::huggingface::utils::{extract_parameter_count, extract_quantization, ollama_model_name};
use crate::core::ollama::models::OllamaModelClient;

const PARALLEL_CHUNKS: usize = 8;
const MIN_CHUNK_SIZE: u64 = 10 * 1024 * 1024; // 10MB

/// Download Manager - coordinates the entire model download process
pub struct DownloadManager;

impl DownloadManager {
    /// Main entry point for downloading a model file
    pub async fn download_model_file(
        model_id: &str,
        filename: &str,
        app_handle: &AppHandle,
    ) -> Result<(), String> {
        let download_id = generate_download_id(model_id, filename);
        let cancel_token = Arc::new(std::sync::atomic::AtomicBool::new(false));
        insert_cancellation_token(download_id.clone(), cancel_token.clone());

        let paths = ModelPaths::new(app_handle, model_id, filename)?;

        if !paths.model_dir.exists() {
            fs::create_dir_all(&paths.model_dir)
                .await
                .map_err(|e| format!("Failed to create model directory: {}", e))?;
        }

        let resume = Self::determine_resume_state(&paths, filename).await;

        let (_file_path, quantization) = Self::perform_download(  // Added underscore
            model_id,
            filename,
            app_handle,
            &paths,
            &cancel_token,
            &download_id,
            resume,
        ).await?;

        // Generate Modelfile and metadata
        let modelfile_path = Self::generate_model_files(
            app_handle,
            model_id,
            filename,
            &paths,
            &quantization,
        ).await?;

        // Create Ollama model
        let ollama_created = Self::create_ollama_model(
            app_handle,
            model_id,
            filename,
            &quantization,
            &modelfile_path,
        ).await;

        // Send completion event
        let model_name = ollama_model_name(model_id, Some(quantization.as_str()));
        send_completion_event(
            app_handle,
            model_id,
            filename,
            paths.file_path.to_str().unwrap_or(""),
            modelfile_path.to_str().unwrap_or(""),
            &quantization,
            &model_name,
            ollama_created,
        );

        remove_cancellation_token(&download_id);
        send_progress(app_handle, model_id, filename, "complete", 100.0, "Download complete!");

        Ok(())
    }

    /// Determine if we should resume a previous download
    async fn determine_resume_state(paths: &ModelPaths, filename: &str) -> bool {
        // Check for chunk files
        for i in 0..PARALLEL_CHUNKS {
            if paths.chunk_path(filename, i).exists() {
                return true;
            }
        }

        // Check for legacy .part file
        if paths.part_path.exists() {
            let _ = tokio::fs::remove_file(&paths.part_path).await;
            return false;
        }

        false
    }

    /// Perform the actual download
    async fn perform_download(
        model_id: &str,
        filename: &str,
        app_handle: &AppHandle,
        paths: &ModelPaths,
        cancel_token: &Arc<std::sync::atomic::AtomicBool>,
        download_id: &str,
        resume: bool,
    ) -> Result<(std::path::PathBuf, String), String> {
        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            model_id, filename
        );

        let client = reqwest::Client::new();

        // Get total file size
        let head_response = client
            .head(&url)
            .header("User-Agent", "SolynApp/1.0")
            .send()
            .await
            .map_err(|e| format!("Failed to get file info: {}", e))?;

        let total_size = head_response
            .content_length()
            .ok_or_else(|| "Failed to get file size".to_string())?;

        // Determine chunk configuration
        let num_chunks = if total_size < MIN_CHUNK_SIZE * 2 {
            std::cmp::min(PARALLEL_CHUNKS, (total_size / MIN_CHUNK_SIZE) as usize + 1)
        } else {
            PARALLEL_CHUNKS
        };
        let chunk_size = total_size / num_chunks as u64;

        // Check if download is already complete
        if resume && paths.file_path.exists() {
            if let Ok(metadata) = fs::metadata(&paths.file_path).await {
                if metadata.len() == total_size {
                    let quantization = extract_quantization(filename)
                        .unwrap_or_else(|| "default".to_string());
                    return Ok((paths.file_path.clone(), quantization));
                }
            }
        }

        // Check if all chunks are complete (previous interrupted combine)
        if resume && are_all_chunks_complete(paths, filename, num_chunks, total_size).await {
            send_progress(app_handle, model_id, filename, "combining", 0.0, "Combining chunks...");
            combine_chunks(paths, filename, num_chunks).await?;
            cleanup_chunks(paths, filename, num_chunks).await;

            let quantization = extract_quantization(filename)
                .unwrap_or_else(|| "default".to_string());
            return Ok((paths.file_path.clone(), quantization));
        }

        // Calculate progress from existing chunks
        let total_downloaded = if resume {
            get_total_downloaded_size(paths, filename, num_chunks).await
        } else {
            0
        };

        let initial_progress = (total_downloaded as f64 / total_size as f64) * 100.0;

        if resume && total_downloaded > 0 {
            send_progress(
                app_handle,
                model_id,
                filename,
                "resuming",
                initial_progress,
                &format!("Resuming download... {:.1}%", initial_progress)
            );
        } else {
            send_progress(app_handle, model_id, filename, "starting", 0.0, "Starting download...");
        }

        // Download chunks in parallel
        Self::download_chunks_parallel(
            app_handle,
            model_id,
            filename,
            paths,
            &url,
            total_size,
            chunk_size,
            num_chunks,
            cancel_token,
            download_id,
        ).await?;

        // All chunks downloaded - combine them
        send_progress(app_handle, model_id, filename, "combining", 0.0, "Combining chunks...");
        combine_chunks(paths, filename, num_chunks).await?;
        cleanup_chunks(paths, filename, num_chunks).await;

        // Verify final file
        if let Ok(metadata) = fs::metadata(&paths.file_path).await {
            if metadata.len() != total_size {
                return Err(format!(
                    "Final file size mismatch: {}/{} bytes",
                    metadata.len(),
                    total_size
                ));
            }
        } else {
            return Err("Failed to verify final file".to_string());
        }

        let quantization = extract_quantization(filename)
            .unwrap_or_else(|| "default".to_string());
        Ok((paths.file_path.clone(), quantization))
    }

    /// Download chunks in parallel with progress tracking
    async fn download_chunks_parallel(
        app_handle: &AppHandle,
        model_id: &str,
        filename: &str,
        paths: &ModelPaths,
        url: &str,
        total_size: u64,
        chunk_size: u64,
        num_chunks: usize,
        cancel_token: &Arc<std::sync::atomic::AtomicBool>,
        download_id: &str,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let mut handles = Vec::with_capacity(num_chunks);
        let cancel_token_clone = cancel_token.clone();

        for i in 0..num_chunks {
            let start_byte = i as u64 * chunk_size;
            let end_byte = if i == num_chunks - 1 {
                total_size - 1
            } else {
                (i as u64 + 1) * chunk_size - 1
            };

            let paths_clone = ModelPaths {
                model_dir: paths.model_dir.clone(),
                file_path: paths.file_path.clone(),
                part_path: paths.part_path.clone(),
            };

            let handle = tokio::spawn({
                let cancel_token_for_task = cancel_token_clone.clone();
                let client_for_task = client.clone();
                let url_for_task = url.to_string();
                let filename_for_task = filename.to_string();

                async move {
                    download_chunk(
                        &url_for_task,
                        &paths_clone,
                        &filename_for_task,
                        i,
                        start_byte,
                        end_byte,
                        &cancel_token_for_task,
                        &client_for_task,
                    ).await
                }
            });

            handles.push((i, handle));
        }

        // Monitor progress and wait for completion
        let mut last_update = tokio::time::Instant::now();
        let update_interval = tokio::time::Duration::from_millis(500);

        while let Some((_, _handle)) = handles.iter_mut().find(|(_, h)| !h.is_finished()) {  // Added underscore
            if cancel_token.load(Ordering::SeqCst) {
                let progress_percent = (get_total_downloaded_size(paths, filename, num_chunks).await as f64 / total_size as f64) * 100.0;
                let progress_rounded = (progress_percent * 10.0).round() / 10.0;
                send_progress(
                    app_handle,
                    model_id,
                    filename,
                    "cancelled",
                    progress_rounded,
                    &format!("Download cancelled at {:.1}%", progress_rounded)
                );
                remove_cancellation_token(download_id);
                return Err("Download cancelled".to_string());
            }

            let now = tokio::time::Instant::now();
            if now - last_update >= update_interval {
                let total_downloaded = get_total_downloaded_size(paths, filename, num_chunks).await;
                let progress_percent = (total_downloaded as f64 / total_size as f64) * 100.0;
                let progress_rounded = (progress_percent * 10.0).round() / 10.0;

                send_progress(
                    app_handle,
                    model_id,
                    filename,
                    "downloading",
                    progress_rounded,
                    &format!("Downloading... {:.1}%", progress_rounded)
                );

                last_update = now;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Collect results
        let mut errors = Vec::new();
        for (i, handle) in handles {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    errors.push(format!("Chunk {}: {}", i, e));
                    log::warn!("Chunk {} failed: {}", i, e);
                }
                Err(e) => {
                    errors.push(format!("Chunk {} task error: {}", i, e));
                    log::warn!("Chunk {} task error: {}", i, e);
                }
            }
        }

        if !errors.is_empty() {
            cleanup_chunks(paths, filename, num_chunks).await;
            return Err(errors.join("; "));
        }

        Ok(())
    }

    /// Generate Modelfile and metadata
    async fn generate_model_files(
        app_handle: &AppHandle,
        model_id: &str,
        filename: &str,
        paths: &ModelPaths,
        quantization: &str,
    ) -> Result<std::path::PathBuf, String> {
        send_progress(
            app_handle,
            model_id,
            filename,
            "generating_modelfile",
            100.0,
            "Generating Modelfile..."
        );

        let config = ModelFileConfig {
            model_id: model_id.to_string(),
            gguf_filename: filename.to_string(),
            model_dir: paths.model_dir.clone(),
        };

        let modelfile_path = match write_modelfile(&paths.model_dir, &config).await {
            Ok(path) => {
                log::info!("Modelfile created at: {:?}", path);
                send_progress(
                    app_handle,
                    model_id,
                    filename,
                    "modelfile_created",
                    100.0,
                    "Modelfile created"
                );
                path
            }
            Err(e) => {
                log::warn!("Failed to create Modelfile: {}", e);
                return Err(format!("Failed to create Modelfile: {}", e));
            }
        };

        // Generate metadata.json
        let parameter_count = extract_parameter_count(filename);
        if let Err(e) = write_metadata(
            &paths.model_dir,
            model_id,
            filename,
            Some(quantization.to_string()),  // Fixed: convert &str to String
            parameter_count,
        ).await {
            log::warn!("Failed to create metadata.json: {}", e);
        }

        Ok(modelfile_path)
    }

    /// Create Ollama model with retry logic
    async fn create_ollama_model(
        app_handle: &AppHandle,
        model_id: &str,
        filename: &str,
        quantization: &str,
        modelfile_path: &std::path::PathBuf,
    ) -> bool {
        send_progress(
            app_handle,
            model_id,
            filename,
            "creating_ollama_model",
            100.0,
            "Creating Ollama model..."
        );

        let model_name = ollama_model_name(model_id, Some(quantization));
        let ollama_client = OllamaModelClient::new();
        let max_retries = 3;
        let mut current_retry = 0;
        let mut last_error = String::new();

        while current_retry < max_retries {
            current_retry += 1;

            if current_retry > 1 {
                let wait_seconds = 2u64.pow(current_retry as u32 - 1);
                send_progress(
                    app_handle,
                    model_id,
                    filename,
                    "retrying_ollama_creation",
                    100.0,
                    &format!(
                        "Retrying Ollama model creation (attempt {}/{})...",
                        current_retry,
                        max_retries
                    )
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(wait_seconds)).await;
            }

            match ollama_client.create_model(&model_name, modelfile_path).await {
                Ok(message) => {
                    log::info!("Ollama model created: {} - {}", model_name, message);
                    send_progress(
                        app_handle,
                        model_id,
                        filename,
                        "ollama_model_created",
                        100.0,
                        &format!("Ollama model '{}' created successfully", model_name)
                    );
                    send_ollama_created_event(
                        app_handle,
                        &model_name,
                        model_id,
                        quantization,
                        current_retry
                    );
                    return true;
                }
                Err(e) => {
                    last_error = e.clone();
                    log::warn!(
                        "Failed to create Ollama model (attempt {}/{}): {}",
                        current_retry,
                        max_retries,
                        e
                    );
                }
            }
        }

        // All retries failed
        log::error!(
            "Failed to create Ollama model after {} attempts: {}",
            max_retries,
            last_error
        );

        let ollama_running = match crate::core::ollama::client::is_ollama_running().await {
            Ok(running) => running,
            Err(_) => false,
        };

        if !ollama_running {
            send_progress(
                app_handle,
                model_id,
                filename,
                "ollama_not_running",
                100.0,
                "Ollama is not running. Model file downloaded but cannot create Ollama model. \
                 Please start Ollama and try importing manually."
            );
        } else {
            send_progress(
                app_handle,
                model_id,
                filename,
                "ollama_creation_failed",
                100.0,
                &format!(
                    "Failed to create Ollama model after {} attempts: {}. \
                     Please try importing manually.",
                    max_retries,
                    last_error
                )
            );
        }

        send_ollama_failed_event(
            app_handle,
            &model_name,
            model_id,
            &last_error,
            max_retries
        );
        false
    }
}

// Convenience function to maintain backward compatibility
pub async fn download_model_file(
    model_id: &str,
    filename: &str,
    app_handle: &AppHandle,
) -> Result<(), String> {
    DownloadManager::download_model_file(model_id, filename, app_handle).await
}