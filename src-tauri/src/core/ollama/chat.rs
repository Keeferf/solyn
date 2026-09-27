use reqwest;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<bool>,
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
    ThinkingChunk(String),
    Done(ChatResponse),
    Error(String),
}

/// Pure NDJSON parser for Ollama's streaming `/api/chat` response.
///
/// `chat_stream` feeds raw body chunks in and forwards the events out; keeping
/// the buffer/accumulation logic I/O-free means chunk-boundary handling can be
/// tested without a server.
#[derive(Default)]
struct StreamAccumulator {
    buffer: String,
    content: String,
    thinking: String,
    final_response: Option<ChatResponse>,
}

impl StreamAccumulator {
    /// Feed a raw body chunk and get the events for any lines it completed. A
    /// partial trailing line stays buffered for the next chunk.
    fn push_chunk(&mut self, text: &str) -> Vec<ChatEvent> {
        self.buffer.push_str(text);
        let mut events = Vec::new();

        while let Some(newline) = self.buffer.find('\n') {
            let line: String = self.buffer.drain(..=newline).collect();
            self.handle_line(line.trim_end_matches(['\r', '\n']), &mut events);
            if matches!(events.last(), Some(ChatEvent::Error(_))) {
                break;
            }
        }
        events
    }

    /// Flush a trailing unterminated line, then emit the terminal `Done` event.
    fn finish(mut self) -> Vec<ChatEvent> {
        let mut events = Vec::new();
        let rest = std::mem::take(&mut self.buffer);
        let line = rest.trim_end_matches(['\r', '\n']);
        if !line.is_empty() {
            self.handle_line(line, &mut events);
        }
        if !matches!(events.last(), Some(ChatEvent::Error(_))) {
            events.push(ChatEvent::Done(self.into_done()));
        }
        events
    }

    fn handle_line(&mut self, line: &str, events: &mut Vec<ChatEvent>) {
        if line.is_empty() {
            return;
        }

        if let Ok(chunk) = serde_json::from_str::<ChatResponse>(line) {
            // Accumulate reasoning and stream it as it arrives
            if let Some(thinking) = chunk.message.thinking.as_deref() {
                if !thinking.is_empty() {
                    self.thinking.push_str(thinking);
                    events.push(ChatEvent::ThinkingChunk(self.thinking.clone()));
                }
            }

            // Accumulate content and send the running total
            if !chunk.message.content.is_empty() {
                self.content.push_str(&chunk.message.content);
                events.push(ChatEvent::MessageChunk(self.content.clone()));
            }

            // Store the final response when done
            if chunk.done {
                self.final_response = Some(chunk);
            }
        } else if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
                events.push(ChatEvent::Error(error.to_string()));
            }
        }
    }

    /// The `done` chunk (with accumulated content) or a synthetic one when the
    /// stream ended without an explicit done.
    fn into_done(self) -> ChatResponse {
        let thinking = if self.thinking.is_empty() {
            None
        } else {
            Some(self.thinking)
        };

        match self.final_response {
            Some(mut chunk) => {
                chunk.message.content = self.content;
                chunk.message.thinking = thinking;
                chunk
            }
            None => ChatResponse {
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: self.content,
                    thinking,
                },
                done: true,
                total_duration: None,
                load_duration: None,
                prompt_eval_count: None,
                prompt_eval_duration: None,
                eval_count: None,
                eval_duration: None,
            },
        }
    }
}

/// Client for Ollama chat functionality
pub struct OllamaChatClient {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaChatClient {
    pub fn new() -> Self {
        Self::with_base_url("http://localhost:11434".to_string())
    }

    /// Construct against an arbitrary base URL (used by tests).
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(300))
                .pool_max_idle_per_host(1) // Keep connection alive for reuse
                .build()
                .unwrap_or_default(),
            base_url,
        }
    }

    /// Check if Ollama is running
    pub async fn check_health(&self) -> Result<bool, String> {
        let response = self
            .client
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
            think: None,
            options,
        };

        let (tx, rx) = mpsc::unbounded_channel();

        let client = self.client.clone();

        tokio::spawn(async move {
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
                        let _ = tx.send(ChatEvent::Error(error_msg));
                        return;
                    }

                    // Get the stream
                    let stream = resp.bytes_stream();
                    let mut stream = Box::pin(stream);
                    let mut accumulator = StreamAccumulator::default();

                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(bytes) => {
                                if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                                    let mut saw_error = false;
                                    for event in accumulator.push_chunk(&text) {
                                        if matches!(event, ChatEvent::Error(_)) {
                                            saw_error = true;
                                        }
                                        let _ = tx.send(event);
                                    }
                                    if saw_error {
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Failed to read chunk: {}", e);
                                let _ = tx.send(ChatEvent::Error(error_msg));
                                return;
                            }
                        }
                    }

                    // Flush any trailing line and emit the final response.
                    for event in accumulator.finish() {
                        let _ = tx.send(event);
                    }
                }
                Err(e) => {
                    let error_msg = format!("Request failed: {}", e);
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
            think: None,
            options,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| format!("Failed to send chat: {}", e))?;

        if response.status().is_success() {
            let chat_response: ChatResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn line(value: serde_json::Value) -> String {
        format!("{}\n", value)
    }

    fn content_events(events: &[ChatEvent]) -> Vec<String> {
        events
            .iter()
            .filter_map(|e| match e {
                ChatEvent::MessageChunk(c) => Some(c.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn request_omits_unset_optional_fields() {
        let request = ChatRequest {
            model: "m".into(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "hi".into(),
                thinking: None,
            }],
            stream: false,
            think: None,
            options: None,
        };
        let value = serde_json::to_value(&request).unwrap();
        assert!(value.get("think").is_none());
        // `options` has no skip_serializing_if, so it stays as an explicit null.
        assert!(value["options"].is_null());
    }

    #[test]
    fn message_omits_none_thinking() {
        let message = ChatMessage {
            role: "assistant".into(),
            content: "hi".into(),
            thinking: None,
        };
        let value = serde_json::to_value(&message).unwrap();
        assert!(value.get("thinking").is_none());
    }

    #[test]
    fn accumulates_content_across_lines() {
        let mut acc = StreamAccumulator::default();
        let mut events = acc.push_chunk(&line(
            json!({"message": {"role": "assistant", "content": "Hel"}, "done": false}),
        ));
        events.extend(
            acc.push_chunk(&line(
                json!({"message": {"role": "assistant", "content": "lo"}, "done": false}),
            )),
        );
        assert_eq!(content_events(&events), ["Hel", "Hello"]);
    }

    #[test]
    fn holds_partial_line_until_it_completes() {
        let mut acc = StreamAccumulator::default();
        let text =
            json!({"message": {"role": "assistant", "content": "hi"}, "done": false}).to_string();

        // Split the JSON in half: nothing parses until the newline arrives.
        let (a, b) = text.split_at(text.len() / 2);
        assert!(acc.push_chunk(a).is_empty());
        assert_eq!(content_events(&acc.push_chunk(&format!("{}\n", b))), ["hi"]);
    }

    #[test]
    fn supports_crlf_and_multiple_lines_per_chunk() {
        let mut acc = StreamAccumulator::default();
        let a = json!({"message": {"role": "assistant", "content": "a"}, "done": false});
        let b = json!({"message": {"role": "assistant", "content": "b"}, "done": false});
        assert_eq!(
            content_events(&acc.push_chunk(&format!("{}\r\n{}\r\n", a, b))),
            ["a", "ab"]
        );
    }

    #[test]
    fn skips_malformed_lines() {
        let mut acc = StreamAccumulator::default();
        assert!(acc.push_chunk("not json\n{\"partial\":\n").is_empty());
    }

    #[test]
    fn surfaces_error_line_and_stops_processing() {
        let mut acc = StreamAccumulator::default();
        let error = json!({"error": "model not found"});
        let late = json!({"message": {"role": "assistant", "content": "late"}, "done": false});

        let events = acc.push_chunk(&format!("{}\n{}\n", error, late));
        assert_eq!(events.len(), 1);
        match &events[0] {
            ChatEvent::Error(message) => assert_eq!(message, "model not found"),
            other => panic!("expected error, got {:?}", other),
        }
    }

    #[test]
    fn accumulates_thinking_separately_from_content() {
        let mut acc = StreamAccumulator::default();
        let events = acc.push_chunk(&line(json!({
            "message": {"role": "assistant", "content": "", "thinking": "why"},
            "done": false,
        })));

        assert_eq!(events.len(), 1);
        match &events[0] {
            ChatEvent::ThinkingChunk(thinking) => assert_eq!(thinking, "why"),
            other => panic!("expected thinking, got {:?}", other),
        }
    }

    #[test]
    fn done_event_carries_accumulated_content_and_stats() {
        let mut acc = StreamAccumulator::default();
        acc.push_chunk(&line(
            json!({"message": {"role": "assistant", "content": "part"}, "done": false}),
        ));

        // The done line carries no content, so it emits no chunk event.
        let events = acc.push_chunk(&line(
            json!({"message": {"role": "assistant", "content": ""}, "done": true, "eval_count": 7}),
        ));
        assert!(events.is_empty());

        let events = acc.finish();
        assert_eq!(events.len(), 1);
        match &events[0] {
            ChatEvent::Done(response) => {
                assert!(response.done);
                assert_eq!(response.message.content, "part");
                assert_eq!(response.eval_count, Some(7));
            }
            other => panic!("expected done, got {:?}", other),
        }
    }

    #[test]
    fn finish_synthesizes_done_when_stream_was_cut_short() {
        let mut acc = StreamAccumulator::default();
        acc.push_chunk(&line(
            json!({"message": {"role": "assistant", "content": "hi"}, "done": false}),
        ));

        let events = acc.finish();
        match &events[0] {
            ChatEvent::Done(response) => {
                assert_eq!(response.message.content, "hi");
                assert_eq!(response.message.role, "assistant");
            }
            other => panic!("expected done, got {:?}", other),
        }
    }

    #[test]
    fn finish_flushes_trailing_line_without_newline() {
        let mut acc = StreamAccumulator::default();
        let text =
            json!({"message": {"role": "assistant", "content": "tail"}, "done": true}).to_string();
        assert!(acc.push_chunk(&text).is_empty());

        let events = acc.finish();
        assert_eq!(content_events(&events), ["tail"]);
        match events.last() {
            Some(ChatEvent::Done(response)) => assert_eq!(response.message.content, "tail"),
            other => panic!("expected done, got {:?}", other),
        }
    }
}
