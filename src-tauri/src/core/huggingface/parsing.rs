use super::utils::{extract_parameter_count, extract_quantization};
use crate::data::huggingface_model_types::GGUFFileInfo;
use serde_json;

pub fn extract_gguf_files(siblings: Option<&Vec<serde_json::Value>>) -> Vec<GGUFFileInfo> {
    let mut gguf_files = Vec::new();

    if let Some(siblings) = siblings {
        for file in siblings {
            let filename = file["rfilename"]
                .as_str()
                .or_else(|| file["filename"].as_str())
                .unwrap_or("");

            if filename.ends_with(".gguf") {
                let size = file["size"]
                    .as_u64()
                    .or_else(|| file["file_size"].as_u64())
                    .or_else(|| file["file"]["size"].as_u64())
                    .unwrap_or(0);

                let model_id = file.get("model_id").and_then(|v| v.as_str()).unwrap_or("");

                let url = if !model_id.is_empty() {
                    format!(
                        "https://huggingface.co/{}/resolve/main/{}",
                        model_id, filename
                    )
                } else {
                    format!("https://huggingface.co/resolve/main/{}", filename)
                };

                let parameter_count = extract_parameter_count(filename);
                let quantization = extract_quantization(filename);

                gguf_files.push(GGUFFileInfo {
                    filename: filename.to_string(),
                    size,
                    url,
                    parameter_count,
                    quantization,
                });
            }
        }
    }

    gguf_files.sort_by(|a, b| b.size.cmp(&a.size));
    gguf_files
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn siblings() -> Vec<serde_json::Value> {
        vec![
            json!({"rfilename": "model-small-Q4_K_M.gguf", "size": 200, "model_id": "author/repo"}),
            json!({"rfilename": "model-big-Q8_0.gguf", "file_size": 500, "model_id": "author/repo"}),
            json!({"filename": "nested.gguf", "file": {"size": 100}}),
            json!({"rfilename": "README.md", "size": 1}),
        ]
    }

    #[test]
    fn none_siblings_yields_empty() {
        assert!(extract_gguf_files(None).is_empty());
    }

    #[test]
    fn extracts_only_gguf_files() {
        let out = extract_gguf_files(Some(&siblings()));
        assert_eq!(out.len(), 3);
        assert!(out.iter().all(|f| f.filename.ends_with(".gguf")));
    }

    #[test]
    fn sorts_by_size_descending() {
        let out = extract_gguf_files(Some(&siblings()));
        let sizes: Vec<u64> = out.iter().map(|f| f.size).collect();
        assert_eq!(sizes, vec![500, 200, 100]);
    }

    #[test]
    fn reads_sizes_from_all_supported_fields() {
        let out = extract_gguf_files(Some(&siblings()));
        let big = out
            .iter()
            .find(|f| f.filename == "model-big-Q8_0.gguf")
            .unwrap();
        assert_eq!(big.size, 500); // file_size
        let nested = out.iter().find(|f| f.filename == "nested.gguf").unwrap();
        assert_eq!(nested.size, 100); // file.size
    }

    #[test]
    fn builds_url_with_and_without_model_id() {
        let out = extract_gguf_files(Some(&siblings()));
        let with_id = out
            .iter()
            .find(|f| f.filename == "model-small-Q4_K_M.gguf")
            .unwrap();
        assert_eq!(
            with_id.url,
            "https://huggingface.co/author/repo/resolve/main/model-small-Q4_K_M.gguf"
        );
        let without_id = out.iter().find(|f| f.filename == "nested.gguf").unwrap();
        assert_eq!(
            without_id.url,
            "https://huggingface.co/resolve/main/nested.gguf"
        );
    }

    #[test]
    fn extracts_parameter_count_and_quantization() {
        let out = extract_gguf_files(Some(&siblings()));
        let small = out
            .iter()
            .find(|f| f.filename == "model-small-Q4_K_M.gguf")
            .unwrap();
        assert_eq!(small.quantization.as_deref(), Some("Q4_K_M"));
        assert_eq!(small.parameter_count, None);
    }
}
