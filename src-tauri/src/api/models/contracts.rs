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

/// Map the frontend's filter string to a `ModelFilter`. Unknown values fall
/// back to the default (most downloads).
pub fn parse_filter(filter: Option<&str>) -> ModelFilter {
    match filter {
        Some("most_downloads") => ModelFilter::MostDownloads,
        Some("most_liked") => ModelFilter::MostLiked,
        Some("recent") => ModelFilter::Recent,
        _ => ModelFilter::default(),
    }
}

/// Cache clearing wants "no filter" for unknown input rather than the default.
pub fn parse_optional_filter(filter: Option<&str>) -> Option<ModelFilter> {
    match filter {
        Some("most_downloads") => Some(ModelFilter::MostDownloads),
        Some("most_liked") => Some(ModelFilter::MostLiked),
        Some("recent") => Some(ModelFilter::Recent),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_filter_keys() {
        assert_eq!(parse_filter(Some("most_downloads")), ModelFilter::MostDownloads);
        assert_eq!(parse_filter(Some("most_liked")), ModelFilter::MostLiked);
        assert_eq!(parse_filter(Some("recent")), ModelFilter::Recent);
    }

    #[test]
    fn unknown_or_missing_filter_falls_back_to_default() {
        assert_eq!(parse_filter(None), ModelFilter::MostDownloads);
        assert_eq!(parse_filter(Some("bogus")), ModelFilter::MostDownloads);
    }

    #[test]
    fn optional_filter_is_none_for_unknown() {
        assert_eq!(parse_optional_filter(None), None);
        assert_eq!(parse_optional_filter(Some("bogus")), None);
        assert_eq!(
            parse_optional_filter(Some("recent")),
            Some(ModelFilter::Recent)
        );
    }
}
