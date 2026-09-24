// src-tauri/src/api/chat/commands.rs
use tauri::{AppHandle, Manager, Emitter};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::contracts::*;
use crate::core::ollama::chat::{OllamaChatClient, ChatMessage, ChatOptions, ChatEvent};
use crate::core::ollama::models::OllamaModelClient;
use crate::data::chat::{ChatDatabase, ChatSession, ChatSessionWithMessages};

pub fn init_chat_state(app: &tauri::App) {
    let ollama_state = OllamaState {
        chat: Arc::new(OllamaChatClient::new()),
    };
    app.manage(Arc::new(Mutex::new(ollama_state)));
    
    // Initialize database state
    let db_state = ChatDbState {
        db: Arc::new(Mutex::new(None)),
    };
    app.manage(db_state);
}

async fn get_db(app_handle: &AppHandle) -> Result<ChatDatabase, String> {
    let db_state = app_handle.state::<ChatDbState>();
    let mut db_guard = db_state.db.lock().await;
    
    if db_guard.is_none() {
        let db = ChatDatabase::new(app_handle)?;
        *db_guard = Some(db);
    }
    
    Ok(db_guard.as_ref().unwrap().clone())
}

#[tauri::command]
pub async fn create_chat_session(
    app_handle: AppHandle,
    request: CreateSessionRequest,
) -> Result<i64, String> {
    let db = get_db(&app_handle).await?;
    let session_id = db.create_session(&request.model_name, request.title.as_deref()).await?;
    Ok(session_id)
}

#[tauri::command]
pub async fn get_chat_sessions(app_handle: AppHandle) -> Result<Vec<ChatSession>, String> {
    let db = get_db(&app_handle).await?;
    let sessions = db.get_sessions().await?;
    Ok(sessions)
}

#[tauri::command]
pub async fn get_chat_session(
    app_handle: AppHandle,
    session_id: i64,
) -> Result<Option<ChatSessionWithMessages>, String> {
    let db = get_db(&app_handle).await?;
    let session = db.get_session_with_messages(session_id).await?;
    Ok(session)
}

#[tauri::command]
pub async fn delete_chat_session(
    app_handle: AppHandle,
    session_id: i64,
) -> Result<(), String> {
    let db = get_db(&app_handle).await?;
    db.delete_session(session_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn update_chat_session_title(
    app_handle: AppHandle,
    session_id: i64,
    title: String,
) -> Result<(), String> {
    let db = get_db(&app_handle).await?;
    db.update_session_title(session_id, &title).await?;
    Ok(())
}

#[tauri::command]
pub async fn add_message_to_session(
    app_handle: AppHandle,
    session_id: i64,
    message: ChatMessageData,
) -> Result<i64, String> {
    let db = get_db(&app_handle).await?;
    let msg_id = db.add_message(
        session_id,
        &message.role,
        &message.content
    ).await?;
    Ok(msg_id)
}

#[tauri::command]
pub async fn send_chat_message(
    app_handle: AppHandle,
    request: ChatRequestData,
) -> Result<String, String> {
    let state = app_handle.state::<Arc<Mutex<OllamaState>>>();
    let state = state.lock().await;
    
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: request.message.clone(),
        }
    ];
    
    let options = ChatOptions {
        temperature: request.temperature,
        top_p: request.top_p,
        top_k: request.top_k,
        num_ctx: request.num_ctx,
        num_predict: request.num_predict,
    };
    
    let response = state.chat.chat_sync(&request.model, messages, Some(options)).await?;
    
    // Save to database if session_id is provided
    if let Some(session_id) = request.session_id {
        let db = get_db(&app_handle).await?;
        // Save user message
        db.add_message(session_id, "user", &request.message).await?;
        // Save assistant response
        db.add_message(session_id, "assistant", &response.message.content).await?;
    }
    
    Ok(response.message.content)
}

#[tauri::command]
pub async fn send_chat_stream(
    app_handle: AppHandle,
    request: ChatStreamData,
) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    
    let state = app_handle.state::<Arc<Mutex<OllamaState>>>();
    let state = state.lock().await;
    
    // First, check if the model exists in Ollama
    let model_client = OllamaModelClient::new();
    let model_exists = model_client.model_exists(&request.model).await?;
    
    if !model_exists {
        let error_msg = format!("Model '{}' not found in Ollama. Please ensure the model is properly installed.", request.model);
        let _ = window.emit("chat-stream-error", json!({ "error": error_msg }));
        return Err(error_msg);
    }
    
    
    let messages: Vec<ChatMessage> = request.messages
        .iter()
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();
    
    let mut receiver = state.chat.chat_stream(&request.model, messages, None).await?;
    
    // Track if we should save to database
    let session_id = request.session_id;
    
    // Clone for use in spawned task
    let app_handle_clone = app_handle.clone();
    let session_id_clone = session_id;
    
    // Process streaming responses and emit events to frontend
    tokio::spawn(async move {
        let mut full_response = String::new();
        
        while let Some(event) = receiver.recv().await {
            match event {
                ChatEvent::MessageChunk(chunk) => {
                    // We now only get one chunk with the full response
                    full_response = chunk.clone();
                    let _ = window.emit("chat-stream-chunk", json!({ "chunk": chunk }));
                }
                ChatEvent::Done(response) => {
                    let _ = window.emit("chat-stream-done", json!({ "response": response }));
                    
                    // Emit complete event with full response
                    let _ = window.emit("chat-stream-complete", json!({ "response": full_response }));
                    
                    // Save to database if we have a session
                    if let Some(sid) = session_id_clone {
                        if let Ok(db) = get_db(&app_handle_clone).await {
                            let _ = db.add_message(sid, "assistant", &full_response).await;
                        }
                    }
                }
                ChatEvent::Error(error) => {
                    let _ = window.emit("chat-stream-error", json!({ "error": error }));
                }
            }
        }
    });
    
    // Save user messages to database
    if let Some(sid) = session_id {
        let db = get_db(&app_handle).await?;
        for msg in request.messages.iter() {
            if msg.role == "user" {
                let _ = db.add_message(sid, "user", &msg.content).await;
            }
        }
    }
    
    Ok(())
}