pub mod ollama;
pub mod huggingface;
pub mod installation;
pub mod platform;

pub use installation::executor::{
    execute_ollama_installation,
    save_installation_log,
    get_installation_recommendation,
};
pub use installation::verifier::{
    quick_verify_ollama,
    is_ollama_ready,
    verify_ollama_with_details,
    OllamaVerificationStatus,
};

// Re-export from ollama
pub use ollama::client::{
    OllamaClient,
    is_ollama_installed,
    is_ollama_running,
    fetch_ollama_version,
    start_ollama,
    get_installation_instructions,
};
pub use ollama::chat::{
    OllamaChatClient,
    ChatMessage,
    ChatEvent,
    ChatOptions,
    ChatRequest,
    ChatResponse,
};
pub use ollama::models::{
    OllamaModelClient,
    OllamaModel,
    OllamaModelList,
};

// Re-export from huggingface
pub use huggingface::{
    download_model_file,
    cancel_download,
    get_installed_models,
    delete_installed_model,
    delete_model_file,
    delete_model_quantization,
    fetch_model_details,
    fetch_hugging_face_models_page,
    search_hugging_face_models,
    clear_model_cache,
    InstalledModel,
    InstalledModelFile,
    generate_modelfile_content,
    write_modelfile,
    ModelFileConfig,
    extract_parameter_count,
    extract_quantization,
};

// Re-export from platform
pub use platform::detector::detect_operating_system;