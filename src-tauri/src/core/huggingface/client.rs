use reqwest;
use serde_json;
use std::time::Duration;
use std::collections::HashMap;
use crate::data::huggingface_model_types::{
    HFModelSummary, HFModelDetails, ModelFilter, SearchModelsResponse
};
use super::cache::Cache;
use super::parsing::extract_gguf_files;

const MAX_MODELS: usize = 100;
const BATCH_SIZE: usize = 100;

// --- Fetch Functions ---

async fn fetch_gguf_models_with_filter(filter: ModelFilter) -> Result<Vec<HFModelSummary>, String> {
    if let Some(models) = Cache::get_gguf_models(&filter) {
        return Ok(models);
    }
    
    let client = reqwest::Client::new();
    let mut all_models = Vec::new();
    let mut page = 1;
    let mut consecutive_empty_pages = 0;
    
    let sort_param = match filter {
        ModelFilter::MostDownloads => "downloads",
        ModelFilter::MostLiked => "likes",
        ModelFilter::Recent => "lastModified",
    };
    
    loop {
        if all_models.len() >= MAX_MODELS {
            break;
        }

        let url = format!(
            "https://huggingface.co/api/models?search=gguf&sort={}&direction=-1&limit={}&page={}",
            sort_param, BATCH_SIZE, page
        );
        
        let response = client
            .get(&url)
            .header("User-Agent", "SolynApp/1.0")
            .header("Cache-Control", "no-cache")
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Hugging Face models: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Hugging Face API error: {}", response.status()));
        }
        
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        let items = if let Some(items_array) = data.as_array() {
            items_array.clone()
        } else if let Some(models_array) = data.get("models").and_then(|v| v.as_array()) {
            models_array.clone()
        } else {
            break;
        };
        
        if items.is_empty() {
            consecutive_empty_pages += 1;
            if consecutive_empty_pages > 2 {
                break;
            }
            page += 1;
            continue;
        }

        consecutive_empty_pages = 0;
        let remaining = MAX_MODELS - all_models.len();
        
        for item in items.iter().take(remaining) {
            if let Some(model) = parse_model_summary(item) {
                all_models.push(model);
            }
        }
        
        page += 1;
        
        if items.len() < BATCH_SIZE {
            break;
        }
        
        if all_models.is_empty() && page > 5 {
            break;
        }
    }
    
    Cache::set_gguf_models(filter, all_models.clone());
    Ok(all_models)
}

async fn search_gguf_models(
    query: &str,
    filter: ModelFilter,
) -> Result<Vec<HFModelSummary>, String> {
    let cache_key = format!("{}:{}", query, filter.as_str());
    if let Some(models) = Cache::get_search_results(&cache_key) {
        return Ok(models);
    }
    
    let client = reqwest::Client::new();
    let mut all_models = Vec::new();
    let mut page = 1;
    let mut consecutive_empty_pages = 0;
    
    let sort_param = match filter {
        ModelFilter::MostDownloads => "downloads",
        ModelFilter::MostLiked => "likes",
        ModelFilter::Recent => "lastModified",
    };
    
    let encoded_query = urlencoding::encode(query);
    
    loop {
        if all_models.len() >= MAX_MODELS {
            break;
        }
        
        let url = format!(
            "https://huggingface.co/api/models?search={}+gguf&sort={}&direction=-1&limit={}&page={}",
            encoded_query, sort_param, BATCH_SIZE, page
        );
        
        let response = client
            .get(&url)
            .header("User-Agent", "SolynApp/1.0")
            .header("Cache-Control", "no-cache")
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("Failed to search Hugging Face models: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Hugging Face API error: {}", response.status()));
        }
        
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        let items = if let Some(items_array) = data.as_array() {
            items_array.clone()
        } else if let Some(models_array) = data.get("models").and_then(|v| v.as_array()) {
            models_array.clone()
        } else {
            break;
        };
        
        if items.is_empty() {
            consecutive_empty_pages += 1;
            if consecutive_empty_pages > 2 {
                break;
            }
            page += 1;
            continue;
        }
        
        consecutive_empty_pages = 0;
        let remaining = MAX_MODELS - all_models.len();
        
        for item in items.iter().take(remaining) {
            if let Some(model) = parse_model_summary(item) {
                all_models.push(model);
            }
        }
        
        page += 1;
        
        if items.len() < BATCH_SIZE {
            break;
        }
        
        if all_models.is_empty() && page > 5 {
            break;
        }
    }
    
    Cache::set_search_results(cache_key, all_models.clone());
    Ok(all_models)
}

// --- Parsing Helper ---

fn parse_model_summary(item: &serde_json::Value) -> Option<HFModelSummary> {
    let id = item["id"].as_str()?.to_string();
    if id.is_empty() {
        return None;
    }
    
    let siblings = item.get("siblings").and_then(|s| s.as_array());
    let has_gguf = siblings
        .map(|s| s.iter().any(|f| {
            f["rfilename"].as_str()
                .or_else(|| f["filename"].as_str())
                .map(|name| name.ends_with(".gguf"))
                .unwrap_or(false)
        }))
        .unwrap_or(false);
    
    if siblings.is_none() || has_gguf {
        let parts: Vec<&str> = id.split('/').collect();
        let author = parts.first().unwrap_or(&"").to_string();
        let name = parts.get(1).unwrap_or(&"").to_string();
        
        let created_at = item["created_at"].as_str()
            .or_else(|| item["createdAt"].as_str())
            .map(|s| s.to_string());
        
        let last_modified = item["last_modified"].as_str()
            .or_else(|| item["lastModified"].as_str())
            .map(|s| s.to_string());
        
        Some(HFModelSummary {
            id: id.clone(),
            model_id: id,
            author,
            name,
            downloads: item["downloads"].as_u64(),
            likes: item["likes"].as_u64(),
            created_at: created_at.clone(),
            last_modified: last_modified.or(created_at),
        })
    } else {
        None
    }
}

// --- Public API Functions ---

pub async fn fetch_hugging_face_models_page(
    page: usize,
    limit: usize,
    filter: &ModelFilter,
) -> Result<Vec<HFModelSummary>, String> {
    let all_models = fetch_gguf_models_with_filter(filter.clone()).await?;
    
    let start = (page - 1) * limit;
    let end = std::cmp::min(start + limit, all_models.len());
    
    if start >= all_models.len() {
        return Ok(Vec::new());
    }
    
    Ok(all_models[start..end].to_vec())
}

pub async fn search_hugging_face_models(
    query: &str,
    page: usize,
    limit: usize,
    filter: &ModelFilter,
) -> Result<SearchModelsResponse, String> {
    if query.trim().is_empty() {
        let all_models = fetch_gguf_models_with_filter(filter.clone()).await?;
        let start = (page - 1) * limit;
        let end = std::cmp::min(start + limit, all_models.len());
        
        let models = if start >= all_models.len() {
            Vec::new()
        } else {
            all_models[start..end].to_vec()
        };
        
        return Ok(SearchModelsResponse {
            models,
            total: all_models.len(),
            has_more: end < all_models.len(),
        });
    }
    
    let all_models = search_gguf_models(query, filter.clone()).await?;
    let start = (page - 1) * limit;
    let end = std::cmp::min(start + limit, all_models.len());
    
    let models = if start >= all_models.len() {
        Vec::new()
    } else {
        all_models[start..end].to_vec()
    };
    
    Ok(SearchModelsResponse {
        models,
        total: all_models.len(),
        has_more: end < all_models.len(),
    })
}

pub async fn get_total_model_count_for_filter(filter: &ModelFilter) -> Result<usize, String> {
    let all_models = fetch_gguf_models_with_filter(filter.clone()).await?;
    Ok(all_models.len())
}

pub async fn get_search_model_count(query: &str, filter: &ModelFilter) -> Result<usize, String> {
    if query.trim().is_empty() {
        return get_total_model_count_for_filter(filter).await;
    }
    let all_models = search_gguf_models(query, filter.clone()).await?;
    Ok(all_models.len())
}

async fn fetch_file_sizes(model_id: &str, filenames: &[String]) -> HashMap<String, u64> {
    let mut size_map = HashMap::new();
    let client = reqwest::Client::new();
    
    let mut tasks = Vec::new();
    
    for filename in filenames {
        let client = client.clone();
        let model_id = model_id.to_string();
        let filename = filename.clone();
        
        let task = tokio::spawn(async move {
            let url = format!("https://huggingface.co/{}/resolve/main/{}", model_id, filename);
            let response = client
                .head(&url)
                .header("User-Agent", "SolynApp/1.0")
                .timeout(Duration::from_secs(10))
                .send()
                .await;
            
            if let Ok(response) = response {
                if let Some(size) = response
                    .headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                {
                    return Some((filename, size));
                }
            }
            None
        });
        
        tasks.push(task);
    }
    
    for task in tasks {
        if let Ok(Some((filename, size))) = task.await {
            size_map.insert(filename, size);
        }
    }
    
    size_map
}

pub async fn fetch_model_details(model_id: &str) -> Result<HFModelDetails, String> {
    if let Some(cached) = Cache::get_model_details(model_id) {
        return Ok(cached);
    }
    
    let client = reqwest::Client::new();
    let url = format!("https://huggingface.co/api/models/{}?full=true", model_id);
    
    let response = client
        .get(&url)
        .header("User-Agent", "SolynApp/1.0")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch model details: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Hugging Face API error: {}", response.status()));
    }
    
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let id = data["id"].as_str().unwrap_or("").to_string();
    if id.is_empty() {
        return Err("Invalid model ID".to_string());
    }
    
    let siblings = data["siblings"].as_array();
    
    let mut gguf_filenames = Vec::new();
    let mut siblings_with_model_id = Vec::new();
    
    if let Some(siblings_vec) = siblings {
        for sibling in siblings_vec {
            let filename = sibling["rfilename"].as_str()
                .or_else(|| sibling["filename"].as_str())
                .unwrap_or("");
            
            if filename.ends_with(".gguf") {
                gguf_filenames.push(filename.to_string());
                
                let mut enhanced = sibling.clone();
                enhanced["model_id"] = serde_json::Value::String(id.clone());
                siblings_with_model_id.push(enhanced);
            }
        }
    }
    
    if !gguf_filenames.is_empty() {
        let size_map = fetch_file_sizes(&id, &gguf_filenames).await;
        
        for sibling in &mut siblings_with_model_id {
            if let Some(filename) = sibling["rfilename"].as_str()
                .or_else(|| sibling["filename"].as_str())
            {
                if let Some(&size) = size_map.get(filename) {
                    sibling["size"] = serde_json::Value::Number(size.into());
                }
            }
        }
    }
    
    let gguf_files = extract_gguf_files(Some(&siblings_with_model_id));
    
    if gguf_files.is_empty() {
        return Err("No GGUF files found in this model repository".to_string());
    }
    
    let parts: Vec<&str> = id.split('/').collect();
    let author = parts.get(0).unwrap_or(&"").to_string();
    let name = parts.get(1).unwrap_or(&"").to_string();
    
    let model = HFModelDetails {
        id: id.clone(),
        model_id: id.clone(),
        author,
        name: name.clone(),
        downloads: data["downloads"].as_u64(),
        likes: data["likes"].as_u64(),
        description: data["description"].as_str().map(|s| s.to_string()),
        gguf_files,
    };
    
    Cache::set_model_details(model_id, model.clone());
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_gguf_model_summary() {
        let item = json!({
            "id": "author/repo",
            "downloads": 10,
            "likes": 3,
            "created_at": "2024-01-01",
            "siblings": [{"rfilename": "model.gguf"}],
        });
        let model = parse_model_summary(&item).unwrap();
        assert_eq!(model.id, "author/repo");
        assert_eq!(model.model_id, "author/repo");
        assert_eq!(model.author, "author");
        assert_eq!(model.name, "repo");
        assert_eq!(model.downloads, Some(10));
        assert_eq!(model.likes, Some(3));
        assert_eq!(model.last_modified.as_deref(), Some("2024-01-01"));
    }

    #[test]
    fn skips_model_without_gguf_siblings() {
        let item = json!({"id": "author/repo", "siblings": [{"rfilename": "README.md"}]});
        assert!(parse_model_summary(&item).is_none());
    }

    #[test]
    fn keeps_model_with_no_siblings() {
        let item = json!({"id": "author/repo"});
        assert!(parse_model_summary(&item).is_some());
    }

    #[test]
    fn skips_empty_id() {
        let item = json!({"id": "", "siblings": [{"rfilename": "m.gguf"}]});
        assert!(parse_model_summary(&item).is_none());
    }

    #[test]
    fn last_modified_falls_back_to_created_at() {
        let item = json!({
            "id": "a/b",
            "created_at": "2024-02-02",
            "siblings": [{"rfilename": "m.gguf"}],
        });
        let model = parse_model_summary(&item).unwrap();
        assert_eq!(model.last_modified.as_deref(), Some("2024-02-02"));
    }

    #[test]
    fn last_modified_prefers_explicit_value() {
        let item = json!({
            "id": "a/b",
            "created_at": "2024-02-02",
            "last_modified": "2024-03-03",
            "siblings": [{"rfilename": "m.gguf"}],
        });
        let model = parse_model_summary(&item).unwrap();
        assert_eq!(model.last_modified.as_deref(), Some("2024-03-03"));
    }
}