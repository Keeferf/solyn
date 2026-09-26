// src/api/models/queries.rs
use serde_json;
use tauri::AppHandle;

use super::contracts::*;
use crate::core::huggingface::{
    fetch_hugging_face_models_page, fetch_model_details as client_fetch_model_details,
    get_installed_models, get_search_model_count, get_total_model_count_for_filter,
    resolve_ollama_model_name, search_hugging_face_models,
};
use crate::core::ollama::models::OllamaModelClient;

#[tauri::command]
pub async fn fetch_huggingface_models_page(
    page: usize,
    limit: Option<usize>,
    filter: Option<String>,
) -> Result<Vec<HFModelSummary>, String> {
    let limit = limit.unwrap_or(20);
    let filter = match filter.as_deref() {
        Some("most_downloads") => ModelFilter::MostDownloads,
        Some("most_liked") => ModelFilter::MostLiked,
        Some("recent") => ModelFilter::Recent,
        _ => ModelFilter::default(),
    };
    fetch_hugging_face_models_page(page, limit, &filter).await
}

#[tauri::command]
pub async fn get_huggingface_model_count(filter: Option<String>) -> Result<usize, String> {
    let filter = match filter.as_deref() {
        Some("most_downloads") => ModelFilter::MostDownloads,
        Some("most_liked") => ModelFilter::MostLiked,
        Some("recent") => ModelFilter::Recent,
        _ => ModelFilter::default(),
    };
    get_total_model_count_for_filter(&filter).await
}

#[tauri::command]
pub async fn fetch_model_details(model_id: String) -> Result<HFModelDetails, String> {
    client_fetch_model_details(&model_id).await
}

#[tauri::command]
pub async fn search_huggingface_models(
    query: String,
    page: usize,
    limit: Option<usize>,
    filter: Option<String>,
) -> Result<SearchModelsResponse, String> {
    let limit = limit.unwrap_or(20);
    let filter = match filter.as_deref() {
        Some("most_downloads") => ModelFilter::MostDownloads,
        Some("most_liked") => ModelFilter::MostLiked,
        Some("recent") => ModelFilter::Recent,
        _ => ModelFilter::default(),
    };
    search_hugging_face_models(&query, page, limit, &filter).await
}

#[tauri::command]
pub async fn get_huggingface_search_count(
    query: String,
    filter: Option<String>,
) -> Result<usize, String> {
    let filter = match filter.as_deref() {
        Some("most_downloads") => ModelFilter::MostDownloads,
        Some("most_liked") => ModelFilter::MostLiked,
        Some("recent") => ModelFilter::Recent,
        _ => ModelFilter::default(),
    };
    get_search_model_count(&query, &filter).await
}

#[tauri::command]
pub async fn get_installed_models_command(
    app_handle: AppHandle,
) -> Result<Vec<InstalledModel>, String> {
    get_installed_models(&app_handle).await
}

#[tauri::command]
pub async fn get_chat_models(app_handle: AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let installed = get_installed_models(&app_handle).await?;

    // Get list of Ollama models
    let ollama_client = OllamaModelClient::new();
    let ollama_models = ollama_client.list_models().await.unwrap_or_default();

    let mut chat_models = Vec::new();

    for model in installed {
        for file in &model.files {
            // Quantization is set by the installed-model scanner; fall back to
            // "default" and let the canonical helper decide on the suffix.
            let quantization = file
                .quantization
                .clone()
                .unwrap_or_else(|| "default".to_string());

            // Resolve the real Ollama tag, tolerating legacy name drift.
            // Example: a model imported before the quantization-extraction fix
            // is registered as `..._Q4_K_` while the file now derives `Q4_K_M`;
            // without the fallback it would be marked unregistered and become
            // unusable in the composer.
            let actual_ollama_name =
                resolve_ollama_model_name(&ollama_models, &model.model_id, &quantization);
            let is_registered = actual_ollama_name.is_some();

            let model_value = serde_json::json!({
                "value": format!("{}:{}", model.model_id, file.filename),
                "label": format!("{} ({})", model.name, quantization),
                "model_id": model.model_id,
                "author": model.author,
                "name": model.name,
                "quantization": quantization,
                "parameter_count": file.parameter_count,
                "filename": file.filename,
                "path": file.path,
                "has_modelfile": file.has_modelfile,
                "size": file.size,
                // The actual Ollama tag to send to /api/chat, or "" if nothing
                // in Ollama matches this file.
                "ollama_model_name": actual_ollama_name.unwrap_or_default(),
                "is_registered": is_registered,
            });
            chat_models.push(model_value);
        }
    }

    Ok(chat_models)
}

#[tauri::command]
pub async fn list_ollama_models(_app_handle: AppHandle) -> Result<Vec<String>, String> {
    let model_client = OllamaModelClient::new();
    model_client.list_models().await
}

#[tauri::command]
pub async fn check_ollama_health(_app_handle: AppHandle) -> Result<bool, String> {
    let model_client = OllamaModelClient::new();
    match model_client.list_models().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
