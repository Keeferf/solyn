// src-tauri/src/api/chat/commands.rs
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use super::contracts::*;
use crate::core::context::{build_context, ContextInput, SessionSettings};
use crate::core::ollama::chat::{ChatEvent, ChatMessage, OllamaChatClient};
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
    let session_id = db
        .create_session(&request.model_name, request.title.as_deref())
        .await?;
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
pub async fn delete_chat_session(app_handle: AppHandle, session_id: i64) -> Result<(), String> {
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
    let msg_id = db
        .add_message(session_id, &message.role, &message.content, None)
        .await?;
    Ok(msg_id)
}

#[tauri::command]
pub async fn send_chat_stream(
    app_handle: AppHandle,
    request: ChatStreamData,
) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    // First, check if the model exists in Ollama
    let model_client = OllamaModelClient::new();
    let model_exists = model_client.model_exists(&request.model).await?;

    if !model_exists {
        let error_msg = format!(
            "Model '{}' not found in Ollama. Please ensure the model is properly installed.",
            request.model
        );
        let _ = window.emit("chat-stream-error", json!({ "error": error_msg }));
        return Err(error_msg);
    }

    // Tell the UI whether this is a cold load (model not in memory) so it can
    // show "Loading model" instead of "Thinking" while waiting for first token.
    let model_loaded = model_client
        .is_model_loaded(&request.model)
        .await
        .unwrap_or(false);
    let _ = window.emit(
        "chat-stream-model-status",
        json!({ "loaded": model_loaded }),
    );

    // The backend owns the transcript: persist settings and the user turn, then
    // load prior history. `prior` excludes the message we just inserted.
    let db = get_db(&app_handle).await?;
    let history = if let Some(sid) = request.session_id {
        let prior = db.get_messages_for_session(sid).await?;
        let settings_json = serde_json::to_string(&request.settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        db.update_session_settings(sid, &settings_json).await?;
        db.add_message(sid, "user", &request.message, None).await?;
        prior
    } else {
        Vec::new()
    };

    let assembled = build_context(ContextInput {
        settings: request.settings.clone(),
        history: history
            .into_iter()
            .map(|m| ChatMessage {
                role: m.role,
                content: m.content,
                thinking: m.thinking,
            })
            .collect(),
        user_message: request.message.clone(),
    });

    let mut receiver = {
        let state = app_handle.state::<Arc<Mutex<OllamaState>>>();
        let state = state.lock().await;
        state
            .chat
            .chat_stream(&request.model, assembled.messages, Some(assembled.options))
            .await?
    };

    let session_id = request.session_id;
    let db_clone = db.clone();

    // Process streaming responses and emit events to frontend
    tokio::spawn(async move {
        let mut full_response = String::new();
        let mut full_thinking = String::new();

        while let Some(event) = receiver.recv().await {
            match event {
                ChatEvent::MessageChunk(chunk) => {
                    // Chunks carry the accumulated content so far
                    full_response = chunk.clone();
                    let _ = window.emit("chat-stream-chunk", json!({ "chunk": chunk }));
                }
                ChatEvent::ThinkingChunk(thinking) => {
                    // Chunks carry the accumulated reasoning so far
                    full_thinking = thinking.clone();
                    let _ = window.emit("chat-stream-thinking", json!({ "chunk": thinking }));
                }
                ChatEvent::Done(response) => {
                    let _ = window.emit("chat-stream-done", json!({ "response": response }));
                    let _ = window.emit(
                        "chat-stream-complete",
                        json!({ "response": full_response, "thinking": full_thinking }),
                    );

                    persist_assistant(&db_clone, session_id, &full_response, &full_thinking).await;
                }
                ChatEvent::Error(error) => {
                    // Keep whatever was generated rather than losing a partial
                    // reply to a dropped stream.
                    persist_assistant(&db_clone, session_id, &full_response, &full_thinking).await;
                    let _ = window.emit("chat-stream-error", json!({ "error": error }));
                }
            }
        }
    });

    Ok(())
}

/// Persist an assistant turn, skipping empty replies.
async fn persist_assistant(
    db: &ChatDatabase,
    session_id: Option<i64>,
    content: &str,
    thinking: &str,
) {
    let Some(sid) = session_id else {
        return;
    };
    if content.is_empty() && thinking.is_empty() {
        return;
    }
    let thinking = if thinking.is_empty() {
        None
    } else {
        Some(thinking)
    };
    let _ = db.add_message(sid, "assistant", content, thinking).await;
}

#[tauri::command]
pub async fn update_chat_session_settings(
    app_handle: AppHandle,
    session_id: i64,
    settings: SessionSettings,
) -> Result<(), String> {
    let db = get_db(&app_handle).await?;
    let settings_json = serde_json::to_string(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    db.update_session_settings(session_id, &settings_json).await
}
