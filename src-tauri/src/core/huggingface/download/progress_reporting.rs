use tauri::{AppHandle, Emitter};

use crate::data::download_state::ModelAcquisitionProgress;

/// Send download progress event
pub fn send_progress(
    app_handle: &AppHandle,
    model_id: &str,
    filename: &str,
    status: &str,
    progress: f64,
    message: &str,
) {
    let progress_msg = ModelAcquisitionProgress {
        model_id: model_id.to_string(),
        filename: filename.to_string(),
        status: status.to_string(),
        progress,
        message: message.to_string(),
    };
    let _ = app_handle.emit("model-download-progress", progress_msg);
}

/// Send completion event with metadata
pub fn send_completion_event(
    app_handle: &AppHandle,
    model_id: &str,
    filename: &str,
    file_path: &str,
    modelfile_path: &str,
    quantization: &str,
    model_name: &str,
    ollama_created: bool,
) {
    let _ = app_handle.emit(
        "model-download-complete",
        &serde_json::json!({
            "model_id": model_id,
            "filename": filename,
            "path": file_path,
            "modelfile_path": modelfile_path,
            "quantization": quantization,
            "ollama_model_name": model_name,
            "ollama_created": ollama_created,
        }),
    );
}

/// Send Ollama creation event
pub fn send_ollama_created_event(
    app_handle: &AppHandle,
    model_name: &str,
    model_id: &str,
    quantization: &str,
    attempt: usize,
) {
    let _ = app_handle.emit(
        "ollama-model-created",
        &serde_json::json!({
            "model_name": model_name,
            "model_id": model_id,
            "quantization": quantization,
            "attempt": attempt,
        }),
    );
}

/// Send Ollama failure event
pub fn send_ollama_failed_event(
    app_handle: &AppHandle,
    model_name: &str,
    model_id: &str,
    error: &str,
    attempts: usize,
) {
    let _ = app_handle.emit(
        "ollama-model-creation-failed",
        &serde_json::json!({
            "model_name": model_name,
            "model_id": model_id,
            "error": error,
            "attempts": attempts,
        }),
    );
}
