pub mod chat;
pub mod client;
pub mod models;

pub use chat::{ChatEvent, ChatMessage, ChatOptions, ChatRequest, ChatResponse, OllamaChatClient};
pub use client::{
    fetch_ollama_version, get_installation_instructions, is_ollama_installed, is_ollama_running,
    start_ollama, OllamaClient,
};
pub use models::{OllamaModel, OllamaModelClient, OllamaModelList};
