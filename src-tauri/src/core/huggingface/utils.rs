use regex;

pub fn extract_parameter_count(filename: &str) -> Option<String> {
    let lower = filename.to_lowercase();

    // Order matters: MoE counts (8x7b), then decimals (1.1b), then plain
    // integers (7b) and millions (125m). Matching a whole pattern instead of
    // `contains("1b")` avoids reporting "1.1b" as "1B".
    if let Some(caps) = regex::Regex::new(r"(\d+)x(\d+)b").unwrap().captures(&lower) {
        return Some(format!("{}x{}B", &caps[1], &caps[2]));
    }
    if let Some(caps) = regex::Regex::new(r"(\d+\.\d+)b").unwrap().captures(&lower) {
        return Some(format!("{}B", &caps[1]));
    }
    if let Some(caps) = regex::Regex::new(r"(\d+)b").unwrap().captures(&lower) {
        return Some(format!("{}B", &caps[1]));
    }
    if let Some(caps) = regex::Regex::new(r"(\d+)m").unwrap().captures(&lower) {
        return Some(format!("{}M", &caps[1]));
    }

    None
}

pub fn extract_quantization(filename: &str) -> Option<String> {
    let name = filename.replace(".gguf", "");
    let patterns = [
        r"IQ[1-4]_[A-Z]{1,2}\b",
        r"Q[2-8]_[0-9K_]*[0-9K]\b",
        r"Q[2-8]_[0-9]",
        r"F[1-9][0-9]?",
        r"q4_k_m",
        r"q5_k_m",
        r"q6_k",
        r"q8_0",
        r"q4_0",
        r"q5_0",
        r"q2_k",
        r"q3_k",
        r"f16",
        r"f32",
    ];
    
    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(&format!(r"(?i){}", pattern)) {
            if let Some(caps) = re.captures(&name) {
                if let Some(matched) = caps.get(0) {
                    let quant = matched.as_str().to_uppercase();
                    let normalized = match quant.as_str() {
                        "Q4_K_M" => "Q4_K_M",
                        "Q5_K_M" => "Q5_K_M",
                        "Q6_K" => "Q6_K",
                        "Q8_0" => "Q8_0",
                        "Q4_0" => "Q4_0",
                        "Q5_0" => "Q5_0",
                        "Q2_K" => "Q2_K",
                        "Q3_K" => "Q3_K",
                        "F16" => "F16",
                        "F32" => "F32",
                        _ => &quant,
                    };
                    return Some(normalized.to_string());
                }
            }
        }
    }
    
    None
}

/// Canonical Ollama model name for a downloaded GGUF file.
///
/// This is the single source of truth for the naming used by the download
/// manager, `get_chat_models`, `generate_modelfile` and model deletion. A
/// missing / "default" quantization means no suffix, so a file without a
/// recognized quantization is registered as `author_model` (never
/// `author_model_default`).
pub fn ollama_model_name(model_id: &str, quantization: Option<&str>) -> String {
    let base = model_id.replace('/', "_");
    match quantization {
        Some(q) if !q.is_empty() && q != "default" => format!("{}_{}", base, q),
        _ => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_count_matches_common_sizes() {
        assert_eq!(extract_parameter_count("llama-2-70b.Q4_K_M.gguf").as_deref(), Some("70B"));
        assert_eq!(extract_parameter_count("vicuna-13b.gguf").as_deref(), Some("13B"));
        assert_eq!(extract_parameter_count("phi-125m.gguf").as_deref(), Some("125M"));
        assert_eq!(extract_parameter_count("wizardlm-3b.gguf").as_deref(), Some("3B"));
        assert_eq!(extract_parameter_count("tinyllama-1.1b.gguf").as_deref(), Some("1.1B"));
        assert_eq!(extract_parameter_count("qwen2.5-3b.gguf").as_deref(), Some("3B"));
        assert_eq!(extract_parameter_count("llava-v1.5-13b.gguf").as_deref(), Some("13B"));
    }

    #[test]
    fn parameter_count_is_none_without_a_size() {
        assert_eq!(extract_parameter_count("model-instruct.gguf"), None);
    }

    #[test]
    fn quantization_extracts_full_k_quant() {
        assert_eq!(
            extract_quantization("Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf").as_deref(),
            Some("Q4_K_M")
        );
    }

    #[test]
    fn quantization_handles_common_variants_case_insensitively() {
        assert_eq!(extract_quantization("model.Q8_0.gguf").as_deref(), Some("Q8_0"));
        assert_eq!(extract_quantization("model.q4_0.gguf").as_deref(), Some("Q4_0"));
        assert_eq!(extract_quantization("model-q5_k_m.gguf").as_deref(), Some("Q5_K_M"));
        assert_eq!(extract_quantization("model-f16.gguf").as_deref(), Some("F16"));
        assert_eq!(extract_quantization("model-f32.gguf").as_deref(), Some("F32"));
        assert_eq!(extract_quantization("model.IQ2_XS.gguf").as_deref(), Some("IQ2_XS"));
    }

    #[test]
    fn quantization_is_none_for_plain_name() {
        assert_eq!(extract_quantization("model.gguf"), None);
    }

    #[test]
    fn ollama_model_name_includes_quantization() {
        assert_eq!(
            ollama_model_name("meta-llama/Llama-3", Some("Q4_K_M")),
            "meta-llama_Llama-3_Q4_K_M"
        );
    }

    #[test]
    fn ollama_model_name_omits_absent_or_default_quantization() {
        assert_eq!(ollama_model_name("a/b", None), "a_b");
        assert_eq!(ollama_model_name("a/b", Some("")), "a_b");
        assert_eq!(ollama_model_name("a/b", Some("default")), "a_b");
    }
}