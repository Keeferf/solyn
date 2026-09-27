pub mod context;
pub mod huggingface;
pub mod installation;
pub mod ollama;
pub mod platform;

pub use installation::executor::{
    execute_ollama_installation, get_installation_recommendation, save_installation_log,
};
pub use installation::verifier::{
    is_ollama_ready, quick_verify_ollama, verify_ollama_with_details, OllamaVerificationStatus,
};

// Re-export from ollama
pub use ollama::chat::{
    ChatEvent, ChatMessage, ChatOptions, ChatRequest, ChatResponse, OllamaChatClient,
};
pub use ollama::client::{
    fetch_ollama_version, get_installation_instructions, is_ollama_installed, is_ollama_running,
    start_ollama, OllamaClient,
};
pub use ollama::models::{OllamaModel, OllamaModelClient, OllamaModelList};

// Re-export from huggingface
pub use huggingface::{
    cancel_download, clear_model_cache, delete_installed_model, delete_model_file,
    delete_model_quantization, download_model_file, extract_parameter_count, extract_quantization,
    fetch_hugging_face_models_page, fetch_model_details, generate_modelfile_content,
    get_installed_models, ollama_model_name, search_hugging_face_models, write_modelfile,
    InstalledModel, InstalledModelFile, ModelFileConfig,
};

// Re-export from platform
pub use platform::detector::detect_operating_system;
