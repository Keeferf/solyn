use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelImportRequest {
    pub model_name: String,
    pub modelfile_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFileResponse {
    pub modelfile_path: String,
    pub ollama_model_name: String,
    pub quantization: String,
}

pub use crate::core::huggingface::InstalledModel;
pub use crate::data::huggingface_model_types::{
    HFModelDetails, HFModelSummary, ModelFilter, SearchModelsResponse,
};
