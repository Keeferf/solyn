use reqwest;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub options: Option<ChatOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatOptions {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub num_ctx: Option<i32>,
    pub num_predict: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub done: bool,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<i32>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<i32>,
    pub eval_duration: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum ChatEvent {
    MessageChunk(String),
    Done(ChatResponse),
    Error(String),
}

/// Client for Ollama chat functionality
pub struct OllamaChatClient {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaChatClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(300))
                .pool_max_idle_per_host(1)  // Keep connection alive for reuse
                .build()
                .unwrap_or_default(),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    /// Check if Ollama is running
    pub async fn check_health(&self) -> Result<bool, String> {
        let response = self.client
            .get(&format!("{}/api/version", self.base_url))
            .timeout(Duration::from_secs(2))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => Ok(true),
            _ => Ok(false),
        }
    }

    /// Send a chat message (streaming)
    pub async fn chat_stream(
        &self,
        model_name: &str,
        messages: Vec<ChatMessage>,
        options: Option<ChatOptions>,
    ) -> Result<mpsc::UnboundedReceiver<ChatEvent>, String> {
        let url = format!("{}/api/chat", self.base_url);
        
        let request = ChatRequest {
            model: model_name.to_string(),
            messages,
            stream: true,
            options,
        };

        let (tx, rx) = mpsc::unbounded_channel();

        let client = self.client.clone();
        let model = model_name.to_string();
        
        tokio::spawn(async move {
            println!("📤 Starting chat stream for model: {}", model);
            
            let response = client
                .post(&url)
                .json(&request)
                .timeout(Duration::from_secs(600))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let error_msg = format!("HTTP error: {}", resp.status());
                        println!("❌ {}", error_msg);
                        let _ = tx.send(ChatEvent::Error(error_msg));
                        return;
                    }

                    // Get the stream
                    let stream = resp.bytes_stream();
                    let mut stream = Box::pin(stream);
                    let mut buffer = String::new();
                    let mut full_content = String::new();
                    let mut final_response: Option<ChatResponse> = None;
                    
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(bytes) => {
                                if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                                    buffer.push_str(&text);
                                    
                                    // Process complete lines
                                    let lines: Vec<String> = buffer
                                        .lines()
                                        .map(|s| s.to_string())
                                        .collect();
                                    
                                    // Keep any incomplete line in buffer
                                    if let Some(last_line) = lines.last() {
                                        if !text.ends_with('\n') && !text.is_empty() {
                                            buffer = last_line.clone();
                                        } else {
                                            buffer.clear();
                                        }
                                    }
                                    
                                    // Process complete lines
                                    for line in lines.iter() {
                                        if line.is_empty() {
                                            continue;
                                        }
                                        if let Ok(chunk) = serde_json::from_str::<ChatResponse>(line) {
                                            // Accumulate content
                                            if !chunk.message.content.is_empty() {
                                                full_content.push_str(&chunk.message.content);
                                                
                                                // ✅ FIX: Send chunk immediately as it arrives
                                                let _ = tx.send(ChatEvent::MessageChunk(full_content.clone()));
                                            }
                                            
                                            // Store the final response when done
                                            if chunk.done {
                                                final_response = Some(chunk);
                                            }
                                        } else {
                                            // Try to parse as error response
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                                                if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
                                                    println!("❌ Ollama error: {}", error);
                                                    let _ = tx.send(ChatEvent::Error(error.to_string()));
                                                    return;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Failed to read chunk: {}", e);
                                println!("❌ {}", error_msg);
                                let _ = tx.send(ChatEvent::Error(error_msg));
                                return;
                            }
                        }
                    }
                    
                    // Send the done event with the complete response
                    if let Some(mut chunk) = final_response {
                        chunk.message.content = full_content;
                        println!("✅ Chat stream completed for model: {}", model);
                        let _ = tx.send(ChatEvent::Done(chunk));
                    } else {
                        // If we didn't get a proper done response, create one
                        let done_response = ChatResponse {
                            message: ChatMessage {
                                role: "assistant".to_string(),
                                content: full_content,
                            },
                            done: true,
                            total_duration: None,
                            load_duration: None,
                            prompt_eval_count: None,
                            prompt_eval_duration: None,
                            eval_count: None,
                            eval_duration: None,
                        };
                        let _ = tx.send(ChatEvent::Done(done_response));
                    }
                }
                Err(e) => {
                    let error_msg = format!("Request failed: {}", e);
                    println!("❌ {}", error_msg);
                    let _ = tx.send(ChatEvent::Error(error_msg));
                }
            }
        });

        Ok(rx)
    }

    /// Send a chat message (non-streaming)
    pub async fn chat_sync(
        &self,
        model_name: &str,
        messages: Vec<ChatMessage>,
        options: Option<ChatOptions>,
    ) -> Result<ChatResponse, String> {
        let url = format!("{}/api/chat", self.base_url);
        
        let request = ChatRequest {
            model: model_name.to_string(),
            messages,
            stream: false,
            options,
        };

        println!("📤 Sending sync chat request to model: {}", model_name);

        let response = self.client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| format!("Failed to send chat: {}", e))?;

        if response.status().is_success() {
            let chat_response: ChatResponse = response.json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            println!("✅ Sync chat completed for model: {}", model_name);
            Ok(chat_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Chat request failed: {}", error_text))
        }
    }
}

impl Default for OllamaChatClient {
    fn default() -> Self {
        Self::new()
    }
}