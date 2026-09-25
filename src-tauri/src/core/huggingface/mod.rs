mod cache;
mod client;
mod download;  
mod parsing;
mod utils;
mod installed;
mod modelfile;

pub use client::{
    fetch_model_details,
    fetch_hugging_face_models_page,
    search_hugging_face_models,
    get_total_model_count_for_filter,
    get_search_model_count,
};
pub use download::{download_model_file, cancel_download};
pub use parsing::extract_gguf_files;
pub use cache::clear_model_cache;
pub use installed::{
    get_installed_models,
    delete_installed_model,
    delete_model_file,
    delete_model_quantization,
    InstalledModel,
    InstalledModelFile
};
pub use modelfile::{
    generate_modelfile_content,
    write_modelfile,
    ModelFileConfig,
    get_modelfile_name
};
pub use utils::{extract_parameter_count, extract_quantization, ollama_model_name};