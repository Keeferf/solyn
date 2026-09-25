use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::data::huggingface_model_types::{HFModelSummary, HFModelDetails, ModelFilter};

static MODEL_DETAILS_CACHE: Lazy<Mutex<HashMap<String, HFModelDetails>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

static GGUF_MODELS_CACHE: Lazy<Mutex<HashMap<ModelFilter, Vec<HFModelSummary>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

static SEARCH_CACHE: Lazy<Mutex<HashMap<String, Vec<HFModelSummary>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct Cache;

impl Cache {
    pub fn get_model_details(model_id: &str) -> Option<HFModelDetails> {
        let cache = MODEL_DETAILS_CACHE.lock().unwrap();
        cache.get(model_id).cloned()
    }

    pub fn set_model_details(model_id: &str, details: HFModelDetails) {
        let mut cache = MODEL_DETAILS_CACHE.lock().unwrap();
        cache.insert(model_id.to_string(), details);
    }

    pub fn get_gguf_models(filter: &ModelFilter) -> Option<Vec<HFModelSummary>> {
        let cache = GGUF_MODELS_CACHE.lock().unwrap();
        cache.get(filter).cloned()
    }

    pub fn set_gguf_models(filter: ModelFilter, models: Vec<HFModelSummary>) {
        let mut cache = GGUF_MODELS_CACHE.lock().unwrap();
        cache.insert(filter, models);
    }

    pub fn get_search_results(key: &str) -> Option<Vec<HFModelSummary>> {
        let cache = SEARCH_CACHE.lock().unwrap();
        cache.get(key).cloned()
    }

    pub fn set_search_results(key: String, models: Vec<HFModelSummary>) {
        let mut cache = SEARCH_CACHE.lock().unwrap();
        cache.insert(key, models);
    }

    pub fn clear_gguf_cache(filter: Option<ModelFilter>) {
        let mut cache = GGUF_MODELS_CACHE.lock().unwrap();
        if let Some(filter) = filter {
            cache.remove(&filter);
        } else {
            cache.clear();
        }
        
        let mut search_cache = SEARCH_CACHE.lock().unwrap();
        search_cache.clear();
    }
}

// Download cancellation tokens
static DOWNLOAD_CANCELLATION_TOKENS: Lazy<Mutex<HashMap<String, Arc<AtomicBool>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn generate_download_id(model_id: &str, filename: &str) -> String {
    format!("{}:{}", model_id, filename)
}

pub fn get_cancellation_token(download_id: &str) -> Option<Arc<AtomicBool>> {
    let tokens = DOWNLOAD_CANCELLATION_TOKENS.lock().unwrap();
    tokens.get(download_id).cloned()
}

pub fn insert_cancellation_token(download_id: String, token: Arc<AtomicBool>) {
    let mut tokens = DOWNLOAD_CANCELLATION_TOKENS.lock().unwrap();
    tokens.insert(download_id, token);
}

pub fn remove_cancellation_token(download_id: &str) -> Option<Arc<AtomicBool>> {
    let mut tokens = DOWNLOAD_CANCELLATION_TOKENS.lock().unwrap();
    tokens.remove(download_id)
}

pub fn clear_model_cache(filter: Option<ModelFilter>) {
    Cache::clear_gguf_cache(filter);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn download_id_is_model_and_filename() {
        assert_eq!(
            generate_download_id("author/repo", "m.gguf"),
            "author/repo:m.gguf"
        );
    }

    #[test]
    fn cancellation_token_lifecycle() {
        let id = "test-cancel-lifecycle-id".to_string();
        remove_cancellation_token(&id);
        assert!(get_cancellation_token(&id).is_none());

        let token = Arc::new(AtomicBool::new(false));
        insert_cancellation_token(id.clone(), token.clone());

        assert!(!get_cancellation_token(&id).unwrap().load(Ordering::SeqCst));
        token.store(true, Ordering::SeqCst);
        assert!(get_cancellation_token(&id).unwrap().load(Ordering::SeqCst));

        assert!(remove_cancellation_token(&id).is_some());
        assert!(get_cancellation_token(&id).is_none());
    }

    #[test]
    fn model_details_cache_roundtrip() {
        let key = "author/unique-cache-roundtrip";
        let details = HFModelDetails {
            id: key.to_string(),
            model_id: key.to_string(),
            author: "author".to_string(),
            name: "unique-cache-roundtrip".to_string(),
            downloads: Some(1),
            likes: Some(2),
            description: None,
            gguf_files: vec![],
        };

        assert!(Cache::get_model_details(key).is_none());
        Cache::set_model_details(key, details);
        let got = Cache::get_model_details(key).unwrap();
        assert_eq!(got.model_id, key);
        assert_eq!(got.likes, Some(2));
    }
}