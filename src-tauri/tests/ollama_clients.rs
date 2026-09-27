//! Integration tests for the Ollama HTTP clients, driven by a dependency-free
//! mock server (a one-shot `TcpListener`). Line-boundary parsing is covered by
//! the unit tests in `core::ollama::chat`; here we verify the reqwest wiring.

use std::io::{Read, Write};
use std::net::TcpListener;

use solyn_lib::core::ollama::chat::{ChatEvent, ChatMessage, OllamaChatClient};
use solyn_lib::core::ollama::client::OllamaClient;
use solyn_lib::core::ollama::models::OllamaModelClient;

struct MockOllama {
    base_url: String,
    _handle: std::thread::JoinHandle<()>,
}

impl MockOllama {
    fn url(&self) -> String {
        self.base_url.clone()
    }
}

/// Serve `body` to the next few connections, then exit. Responses use
/// `Connection: close` so reqwest doesn't try to reuse the socket.
fn spawn_mock(status: &'static str, content_type: &'static str, body: &'static str) -> MockOllama {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = std::thread::spawn(move || {
        for _ in 0..4 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            // Drain whatever the client sent; we never inspect the request.
            let mut buf = [0u8; 8192];
            let _ = stream.read(&mut buf);

            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
                len = body.len(),
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    });

    MockOllama {
        base_url: format!("http://{}", addr),
        _handle: handle,
    }
}

/// A port that is almost certainly closed, for connection-refused paths.
fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn msg(role: &str, content: &str) -> ChatMessage {
    ChatMessage {
        role: role.to_string(),
        content: content.to_string(),
        thinking: None,
    }
}

// --- OllamaModelClient: /api/tags ---

#[tokio::test]
async fn list_models_parses_names() {
    let server = spawn_mock(
        "200 OK",
        "application/json",
        r#"{"models":[
            {"name":"llama3:latest","modified_at":"2024-01-01","size":100},
            {"name":"qwen2.5:7b","modified_at":"2024-02-02","size":200}
        ]}"#,
    );
    let client = OllamaModelClient::with_base_url(server.url());

    let models = client.list_models().await.unwrap();
    assert_eq!(models, ["llama3:latest", "qwen2.5:7b"]);
}

#[tokio::test]
async fn list_models_handles_empty_list() {
    let server = spawn_mock("200 OK", "application/json", r#"{"models":[]}"#);
    let client = OllamaModelClient::with_base_url(server.url());

    assert!(client.list_models().await.unwrap().is_empty());
}

#[tokio::test]
async fn list_models_errors_on_server_error() {
    let server = spawn_mock("500 Internal Server Error", "text/plain", "nope");
    let client = OllamaModelClient::with_base_url(server.url());

    assert!(client.list_models().await.is_err());
}

#[tokio::test]
async fn list_models_errors_on_malformed_body() {
    let server = spawn_mock("200 OK", "application/json", "{not json");
    let client = OllamaModelClient::with_base_url(server.url());

    assert!(client.list_models().await.is_err());
}

#[tokio::test]
async fn model_exists_matches_name() {
    let server = spawn_mock(
        "200 OK",
        "application/json",
        r#"{"models":[{"name":"llama3:latest","modified_at":"x","size":1}]}"#,
    );
    let client = OllamaModelClient::with_base_url(server.url());

    assert!(client.model_exists("llama3:latest").await.unwrap());
    assert!(!client.model_exists("missing").await.unwrap());
}

#[tokio::test]
async fn delete_model_reports_success_and_failure() {
    let ok = spawn_mock("200 OK", "application/json", "{}");
    assert!(
        OllamaModelClient::with_base_url(ok.url())
            .delete_model("m")
            .await
            .is_ok()
    );

    let fail = spawn_mock("500 Internal Server Error", "text/plain", "boom");
    assert!(
        OllamaModelClient::with_base_url(fail.url())
            .delete_model("m")
            .await
            .is_err()
    );
}

// --- OllamaModelClient: /api/ps (warm-model detection) ---

#[tokio::test]
async fn is_model_loaded_matches_untagged_request_against_tagged_model() {
    let server = spawn_mock(
        "200 OK",
        "application/json",
        r#"{"models":[{"name":"llama3:latest"}]}"#,
    );
    let client = OllamaModelClient::with_base_url(server.url());

    assert!(client.is_model_loaded("llama3").await.unwrap());
}

#[tokio::test]
async fn is_model_loaded_is_false_for_a_different_model() {
    let server = spawn_mock(
        "200 OK",
        "application/json",
        r#"{"models":[{"name":"llama3:latest"}]}"#,
    );
    let client = OllamaModelClient::with_base_url(server.url());

    assert!(!client.is_model_loaded("mistral").await.unwrap());
}

// --- OllamaClient: version + health ---

#[tokio::test]
async fn get_version_parses_json() {
    let server = spawn_mock("200 OK", "application/json", r#"{"version":"0.5.1"}"#);
    let client = OllamaClient::with_base_url(server.url());

    assert_eq!(client.get_version().await.unwrap(), "0.5.1");
}

#[tokio::test]
async fn get_version_falls_back_to_unknown_when_field_missing() {
    let server = spawn_mock("200 OK", "application/json", "{}");
    let client = OllamaClient::with_base_url(server.url());

    assert_eq!(client.get_version().await.unwrap(), "unknown");
}

#[tokio::test]
async fn get_version_errors_on_server_error() {
    let server = spawn_mock("500 Internal Server Error", "text/plain", "down");
    let client = OllamaClient::with_base_url(server.url());

    assert!(client.get_version().await.is_err());
}

#[tokio::test]
async fn check_health_is_true_on_success_and_false_on_error() {
    let ok = spawn_mock("200 OK", "application/json", r#"{"version":"1"}"#);
    assert!(
        OllamaClient::with_base_url(ok.url())
            .check_health()
            .await
            .unwrap()
    );

    let fail = spawn_mock("500 Internal Server Error", "text/plain", "down");
    assert!(
        !OllamaClient::with_base_url(fail.url())
            .check_health()
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn check_health_is_false_when_unreachable() {
    let client = OllamaClient::with_base_url(format!("http://127.0.0.1:{}", unused_port()));
    assert!(!client.check_health().await.unwrap());
}

// --- OllamaChatClient ---

#[tokio::test]
async fn chat_sync_parses_response() {
    let server = spawn_mock(
        "200 OK",
        "application/json",
        r#"{"message":{"role":"assistant","content":"hello"},"done":true,"eval_count":3}"#,
    );
    let client = OllamaChatClient::with_base_url(server.url());

    let response = client
        .chat_sync("m", vec![msg("user", "hi")], None)
        .await
        .unwrap();
    assert_eq!(response.message.content, "hello");
    assert_eq!(response.eval_count, Some(3));
}

#[tokio::test]
async fn chat_sync_surfaces_error_body() {
    let server = spawn_mock("500 Internal Server Error", "text/plain", "model exploded");
    let client = OllamaChatClient::with_base_url(server.url());

    let error = client
        .chat_sync("m", vec![msg("user", "hi")], None)
        .await
        .unwrap_err();
    assert!(error.contains("model exploded"), "got: {error}");
}

#[tokio::test]
async fn chat_stream_emits_accumulated_chunks_then_done() {
    let body = concat!(
        r#"{"message":{"role":"assistant","content":"Hel"},"done":false}"#,
        "\n",
        r#"{"message":{"role":"assistant","content":"lo"},"done":false}"#,
        "\n",
        r#"{"message":{"role":"assistant","content":""},"done":true,"eval_count":5}"#,
        "\n",
    );
    let server = spawn_mock("200 OK", "application/x-ndjson", body);
    let client = OllamaChatClient::with_base_url(server.url());

    let mut rx = client
        .chat_stream("m", vec![msg("user", "hi")], None)
        .await
        .unwrap();

    let mut chunks = Vec::new();
    let mut done = None;
    while let Some(event) = rx.recv().await {
        match event {
            ChatEvent::MessageChunk(chunk) => chunks.push(chunk),
            ChatEvent::Done(response) => {
                done = Some(response);
                break;
            }
            ChatEvent::ThinkingChunk(_) => {}
            ChatEvent::Error(error) => panic!("unexpected stream error: {error}"),
        }
    }

    assert_eq!(chunks, ["Hel", "Hello"]);
    let done = done.expect("a done event");
    assert!(done.done);
    assert_eq!(done.message.content, "Hello");
    assert_eq!(done.eval_count, Some(5));
}

#[tokio::test]
async fn chat_stream_reports_http_error() {
    let server = spawn_mock("500 Internal Server Error", "text/plain", "boom");
    let client = OllamaChatClient::with_base_url(server.url());

    let mut rx = client
        .chat_stream("m", vec![msg("user", "hi")], None)
        .await
        .unwrap();

    match rx.recv().await {
        Some(ChatEvent::Error(error)) => assert!(error.contains("500"), "got: {error}"),
        other => panic!("expected stream error, got {other:?}"),
    }
}

#[tokio::test]
async fn chat_stream_reports_inline_error_line() {
    let body = concat!(r#"{"error":"model not found"}"#, "\n");
    let server = spawn_mock("200 OK", "application/x-ndjson", body);
    let client = OllamaChatClient::with_base_url(server.url());

    let mut rx = client
        .chat_stream("m", vec![msg("user", "hi")], None)
        .await
        .unwrap();

    match rx.recv().await {
        Some(ChatEvent::Error(error)) => assert_eq!(error, "model not found"),
        other => panic!("expected stream error, got {other:?}"),
    }
}
