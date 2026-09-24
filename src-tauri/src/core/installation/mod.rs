pub mod executor;
pub mod verifier;

pub use executor::{
    execute_ollama_installation,
    execute_ollama_update, 
    save_installation_log,
    get_installation_recommendation,
};
pub use verifier::{
    quick_verify_ollama,
    is_ollama_ready,
    verify_ollama_with_details,
    OllamaVerificationStatus,
};