pub mod api;
pub mod core;
pub mod data;
pub mod events;
pub mod helpers;

pub use data::download_state::*;
pub use data::huggingface_model_types::*;

use tauri;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create status monitor
    let status_monitor = events::OllamaStatusMonitor::new();

    tauri::Builder::default()
        .setup(move |app| {
            // Initialize chat state
            api::chat::commands::init_chat_state(app);

            // Start the status monitor using Tauri's async runtime
            let app_handle = app.handle().clone();
            let monitor = status_monitor.clone();
            tauri::async_runtime::spawn(async move {
                monitor.start(app_handle).await;
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Ollama commands
            api::ollama::commands::download_ollama,
            api::ollama::commands::start_ollama_service,
            api::ollama::commands::update_ollama,
            api::ollama::commands::refresh_ollama_status,
            api::ollama::queries::check_ollama_installed,
            api::ollama::queries::check_ollama_running,
            api::ollama::queries::get_ollama_status,
            api::ollama::queries::get_ollama_version,
            api::ollama::queries::get_install_info,
            // Platform commands - Queries
            api::platform::queries::get_platform_info,
            api::platform::queries::get_platform_info_detailed,
            api::platform::queries::get_system_resources,
            api::platform::queries::get_app_version,
            api::platform::queries::get_app_data_path,
            // Platform commands - Commands
            api::platform::commands::open_path,
            api::platform::commands::open_url,
            api::platform::commands::copy_to_clipboard,
            api::platform::commands::restart_app,
            // Model queries
            api::models::queries::fetch_huggingface_models_page,
            api::models::queries::get_huggingface_model_count,
            api::models::queries::fetch_model_details,
            api::models::queries::search_huggingface_models,
            api::models::queries::get_huggingface_search_count,
            api::models::queries::get_installed_models_command,
            api::models::queries::get_chat_models,
            api::models::queries::list_ollama_models,
            api::models::queries::check_ollama_health,
            // Model commands
            api::models::commands::clear_models_cache,
            api::models::commands::download_huggingface_model,
            api::models::commands::cancel_huggingface_download,
            api::models::commands::delete_installed_model_command,
            api::models::commands::delete_model_file_command,
            api::models::commands::delete_model_quantization_command,
            api::models::commands::generate_modelfile,
            api::models::commands::create_ollama_model,
            api::models::commands::delete_ollama_model,
            api::models::commands::debug_ollama_models,
            api::models::commands::test_chat_with_model,
            // Chat history commands
            api::chat::commands::create_chat_session,
            api::chat::commands::get_chat_sessions,
            api::chat::commands::get_chat_session,
            api::chat::commands::delete_chat_session,
            api::chat::commands::update_chat_session_title,
            api::chat::commands::update_chat_session_settings,
            api::chat::commands::add_message_to_session,
            // Chat commands
            api::chat::commands::send_chat_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
