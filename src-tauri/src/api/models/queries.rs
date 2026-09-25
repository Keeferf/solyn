// src/api/models/queries.rs
use tauri::AppHandle;
use serde_json;

use super::contracts::*;
use crate::core::huggingface::{
    fetch_hugging_face_models_page,
    get_total_model_count_for_filter,
    search_hugging_face_models,
    get_search_model_count,
    get_installed_models,
    ollama_model_name,
    fetch_model_details as client_fetch_model_details,
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
pub async fn get_huggingface_model_count(
    filter: Option<String>,
) -> Result<usize, String> {
    let filter = match filter.as_deref() {
        Some("most_downloads") => ModelFilter::MostDownloads,
        Some("most_liked") => ModelFilter::MostLiked,
        Some("recent") => ModelFilter::Recent,
        _ => ModelFilter::default(),
    };
    get_total_model_count_for_filter(&filter).await
}

#[tauri::command]
pub async fn fetch_model_details(
    model_id: String,
) -> Result<HFModelDetails, String> {
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
pub async fn get_chat_models(
    app_handle: AppHandle,
) -> Result<Vec<serde_json::Value>, String> {
    let installed = get_installed_models(&app_handle).await?;
    
    // Get list of Ollama models
    let ollama_client = OllamaModelClient::new();
    let ollama_models = match ollama_client.list_models().await {
        Ok(models) => {
            models
        },
        Err(_) => {
            Vec::new()
        }
    };
    
    let mut chat_models = Vec::new();
    
    for model in installed {
        for file in &model.files {
            // Quantization is set by the installed-model scanner; fall back to
            // "default" and let the canonical helper decide on the suffix.
            let quantization = file
                .quantization
                .clone()
                .unwrap_or_else(|| "default".to_string());
            
            // Base Ollama model name (without :latest), identical to the name
            // created during download.
            let base_ollama_name = ollama_model_name(&model.model_id, Some(&quantization));
            
            // Check if this model exists in Ollama (check both with and without :latest)
            let is_registered = ollama_models.iter().any(|m| {
                // Check exact match
                if m == &base_ollama_name {
                    return true;
                }
                // Check with :latest suffix (Ollama adds this automatically)
                if m == &format!("{}:latest", base_ollama_name) {
                    return true;
                }
                // Check if the model name is in the list (partial match)
                m.starts_with(&base_ollama_name)
            });
            
            // Determine the actual Ollama model name to use
            // Prefer the one with :latest if it exists, otherwise use the base name
            let actual_ollama_name = if ollama_models.iter().any(|m| m == &format!("{}:latest", base_ollama_name)) {
                format!("{}:latest", base_ollama_name)
            } else if ollama_models.iter().any(|m| m == &base_ollama_name) {
                base_ollama_name.clone()
            } else {
                // If not found, use the base name and let the chat fail with a clear error
                base_ollama_name.clone()
            };
            
            
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
                // Use the actual ollama name (with :latest if it exists)
                "ollama_model_name": if is_registered { actual_ollama_name } else { String::new() },
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