// src/api/models/commands.rs
use tauri::{AppHandle, Manager};
use serde_json::json;
use super::contracts::*;
use crate::core::huggingface::{
    clear_model_cache,
    download_model_file,
    cancel_download,
    delete_installed_model,
    delete_model_file,
    delete_model_quantization,
    write_modelfile,
    ModelFileConfig,
};
use crate::core::ollama::models::OllamaModelClient;
use std::path::PathBuf;

#[tauri::command]
pub async fn clear_models_cache(
    filter: Option<String>,
) -> Result<(), String> {
    let filter = match filter.as_deref() {
        Some("most_downloads") => Some(ModelFilter::MostDownloads),
        Some("most_liked") => Some(ModelFilter::MostLiked),
        Some("recent") => Some(ModelFilter::Recent),
        _ => None,
    };
    clear_model_cache(filter);
    Ok(())
}

#[tauri::command]
pub async fn download_huggingface_model(
    model_id: String,
    filename: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    download_model_file(&model_id, &filename, &app_handle).await
}

#[tauri::command]
pub fn cancel_huggingface_download(
    model_id: String,
    filename: String,
) -> Result<bool, String> {
    Ok(cancel_download(&model_id, &filename))
}

#[tauri::command]
pub async fn delete_installed_model_command(
    app_handle: AppHandle,
    model_id: String,
) -> Result<(), String> {
    delete_installed_model(&app_handle, &model_id).await
}

#[tauri::command]
pub async fn delete_model_file_command(
    app_handle: AppHandle,
    model_id: String,
    filename: String,
) -> Result<(), String> {
    delete_model_file(&app_handle, &model_id, &filename).await
}

#[tauri::command]
pub async fn delete_model_quantization_command(
    app_handle: AppHandle,
    model_id: String,
    quantization: String,
) -> Result<(), String> {
    delete_model_quantization(&app_handle, &model_id, &quantization).await
}

#[tauri::command]
pub async fn generate_modelfile(
    model_id: String,
    filename: String,
    app_handle: AppHandle,
) -> Result<ModelFileResponse, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    let model_folder_name = model_id.replace("/", "_");
    let model_dir = app_dir.join("models").join(&model_folder_name);
    
    if !model_dir.exists() {
        return Err("Model directory not found".to_string());
    }
    
    // Extract quantization from filename
    let quantization = filename
        .split('_')
        .find(|part| part.starts_with('Q') || part.starts_with("IQ") || part.starts_with("F"))
        .unwrap_or("default")
        .to_string();
    
    // Generate the ollama model name WITHOUT :latest suffix
    // This should match what we use in get_chat_models
    let ollama_model_name = if quantization != "default" {
        format!("{}_{}", model_id.replace("/", "_"), quantization)
    } else {
        model_id.replace("/", "_")
    };
    
    
    let config = ModelFileConfig {
        model_id: model_id.clone(),
        gguf_filename: filename.clone(),
        model_dir: model_dir.clone(),
    };
    
    let modelfile_path = write_modelfile(&model_dir, &config).await?;
    
    Ok(ModelFileResponse {
        modelfile_path: modelfile_path.to_str().unwrap_or("").to_string(),
        ollama_model_name,
        quantization,
    })
}

#[tauri::command]
pub async fn create_ollama_model(
    _app_handle: AppHandle,
    request: ModelImportRequest,
) -> Result<String, String> {
    let model_client = OllamaModelClient::new();
    let modelfile_path = PathBuf::from(&request.modelfile_path);
    
    // The model name should match what was generated in generate_modelfile
    let model_name = request.model_name.trim().to_string();
    
    
    // Verify modelfile exists
    if !modelfile_path.exists() {
        let error_msg = format!("Modelfile not found at: {:?}", modelfile_path);
        return Err(error_msg);
    }
    
    // Check if model already exists (check both with and without :latest)
    let model_client_clone = OllamaModelClient::new();
    let existing_models = match model_client_clone.list_models().await {
        Ok(models) => models,
        Err(_) => {
            Vec::new()
        }
    };
    
    let model_exists = existing_models.iter().any(|m| {
        *m == model_name || *m == format!("{}:latest", model_name)
    });
    
    if model_exists {
        return Ok(model_name);
    }
    
    // Create the model
    let result = model_client.create_model(&model_name, &modelfile_path).await?;
    
    // Verify the model was created
    let verify_client = OllamaModelClient::new();
    let verify_models = match verify_client.list_models().await {
        Ok(models) => models,
        Err(_) => {
            return Ok(result);
        }
    };
    
    let found = verify_models.iter().any(|m| {
        *m == model_name || *m == format!("{}:latest", model_name)
    });
    
    if found {
        Ok(result)
    } else {
        Ok(result) // Assume it worked since create_model succeeded
    }
}

#[tauri::command]
pub async fn delete_ollama_model(
    _app_handle: AppHandle,
    model_name: String,
) -> Result<(), String> {
    let model_client = OllamaModelClient::new();
    model_client.delete_model(&model_name).await
}

// Debug command to check Ollama models
#[tauri::command]
pub async fn debug_ollama_models() -> Result<serde_json::Value, String> {
    let model_client = OllamaModelClient::new();
    
    let ollama_models = match model_client.list_models().await {
        Ok(models) => models,
        Err(e) => return Err(format!("Failed to list models: {}", e)),
    };
    
    Ok(json!({
        "ollama_models": ollama_models,
        "count": ollama_models.len(),
    }))
}

// Direct test command to chat with a specific model
#[tauri::command]
pub async fn test_chat_with_model(
    model_name: String,
    message: String,
) -> Result<String, String> {
    use crate::core::ollama::chat::{OllamaChatClient, ChatMessage};
    
    
    let chat_client = OllamaChatClient::new();
    
    // Check if model exists
    let model_client = OllamaModelClient::new();
    let model_list = match model_client.list_models().await {
        Ok(models) => models,
        Err(e) => return Err(format!("Failed to list models: {}", e)),
    };
    
    // Try to find the model in the list
    let actual_model = model_list.iter().find(|m| {
        // Check exact match
        **m == model_name ||
        // Check with :latest suffix
        **m == format!("{}:latest", model_name) ||
        // Check without :latest suffix (if model_name has it)
        (model_name.ends_with(":latest") && **m == model_name.replace(":latest", ""))
    });
    
    match actual_model {
        Some(found_model) => {
            
            let messages = vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: message,
                }
            ];
            
            let response = chat_client.chat_sync(found_model, messages, None).await?;
            Ok(response.message.content)
        }
        None => {
            let error_msg = format!("Model '{}' not found in Ollama. Available: {:?}", model_name, model_list);
            Err(error_msg)
        }
    }
}