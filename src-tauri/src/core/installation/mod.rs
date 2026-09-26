pub mod executor;
pub mod verifier;

pub use executor::{
    execute_ollama_installation, execute_ollama_update, get_installation_recommendation,
    save_installation_log,
};
pub use verifier::{
    is_ollama_ready, quick_verify_ollama, verify_ollama_with_details, OllamaVerificationStatus,
};
