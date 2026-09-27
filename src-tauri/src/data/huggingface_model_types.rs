// src/data/huggingface_model_types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HFModelSummary {
    pub id: String,
    pub model_id: String,
    pub author: String,
    pub name: String,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
    pub created_at: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HFModelDetails {
    pub id: String,
    pub model_id: String,
    pub author: String,
    pub name: String,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
    pub description: Option<String>,
    pub gguf_files: Vec<GGUFFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGUFFileInfo {
    pub filename: String,
    pub size: u64,
    pub url: String,
    #[serde(default)]
    pub parameter_count: Option<String>,
    #[serde(default)]
    pub quantization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelFilter {
    MostDownloads,
    MostLiked,
    Recent,
}

impl Default for ModelFilter {
    fn default() -> Self {
        Self::MostDownloads
    }
}

impl ModelFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelFilter::MostDownloads => "downloads",
            ModelFilter::MostLiked => "likes",
            ModelFilter::Recent => "lastModified",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ModelFilter::MostDownloads => "Most Downloads",
            ModelFilter::MostLiked => "Most Liked",
            ModelFilter::Recent => "Recent",
        }
    }
}

// Add search request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchModelsRequest {
    pub query: String,
    pub filter: ModelFilter,
    pub page: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchModelsResponse {
    pub models: Vec<HFModelSummary>,
    pub total: usize,
    pub has_more: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter_is_most_downloads() {
        assert_eq!(ModelFilter::default(), ModelFilter::MostDownloads);
    }

    #[test]
    fn filter_sort_keys() {
        assert_eq!(ModelFilter::MostDownloads.as_str(), "downloads");
        assert_eq!(ModelFilter::MostLiked.as_str(), "likes");
        assert_eq!(ModelFilter::Recent.as_str(), "lastModified");
    }

    #[test]
    fn filter_display_names() {
        assert_eq!(ModelFilter::MostDownloads.display_name(), "Most Downloads");
        assert_eq!(ModelFilter::MostLiked.display_name(), "Most Liked");
        assert_eq!(ModelFilter::Recent.display_name(), "Recent");
    }
}
