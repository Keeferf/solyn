// src-tauri/src/api/chat/contracts.rs
use crate::core::context::SessionSettings;
use crate::core::ollama::chat::OllamaChatClient;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

// Chat state
pub struct OllamaState {
    pub chat: Arc<OllamaChatClient>,
}

// Database state
#[derive(Clone)]
pub struct ChatDbState {
    pub db: Arc<Mutex<Option<crate::data::chat::ChatDatabase>>>,
}

// Request/Response structures for chat commands
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub model_name: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatStreamData {
    pub session_id: Option<i64>,
    pub model: String,
    pub message: String,
    #[serde(default)]
    pub settings: SessionSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatMessageData {
    pub role: String,
    pub content: String,
}
