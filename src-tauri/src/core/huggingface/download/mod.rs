// src/core/huggingface/download/mod.rs

mod chunking_logic;
mod manager;
mod paths;
mod progress_reporting;

pub use manager::download_model_file;

/// Cancel an ongoing download by model_id and filename
pub fn cancel_download(model_id: &str, filename: &str) -> bool {
    let download_id = crate::core::huggingface::cache::generate_download_id(model_id, filename);
    if let Some(token) = crate::core::huggingface::cache::get_cancellation_token(&download_id) {
        token.store(true, std::sync::atomic::Ordering::SeqCst); // Fixed: swapped arguments
        true
    } else {
        false
    }
}
