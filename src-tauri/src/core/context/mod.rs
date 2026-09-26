//! Context assembly: the single place that turns a session's settings, prior
//! history, and the new user message into the exact payload sent to Ollama.
//!
//! Every later feature (code mode, attachments, tools, RAG, context tuning)
//! plugs into `ContextInput` rather than growing a new pipeline. Keeping this in
//! Rust means the token budget is authoritative and the agent loop (Phase 4) can
//! re-assemble server-side without round-tripping through the webview.

mod budget;
mod prompts;

pub use budget::{estimate_messages_tokens, estimate_tokens, trim_to_budget, DEFAULT_NUM_CTX};
pub use prompts::compose_system_prompt;

use serde::{Deserialize, Serialize};

use crate::core::ollama::chat::{ChatMessage, ChatOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Chat,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptionOverrides {
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub num_ctx: Option<i32>,
    #[serde(default)]
    pub num_predict: Option<i32>,
}

/// Per-session configuration. Persisted as JSON on `chat_sessions.settings`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionSettings {
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub code: bool,
    #[serde(default)]
    pub web: bool,
    #[serde(default)]
    pub options: OptionOverrides,
}

pub struct ContextInput {
    pub settings: SessionSettings,
    pub history: Vec<ChatMessage>,
    pub user_message: String,
}

pub struct AssembledContext {
    pub messages: Vec<ChatMessage>,
    pub options: ChatOptions,
}

/// Resolve the context window. Explicit setting wins; otherwise the fallback.
/// Phase 6 fills in per-model trained-window and hardware caps here.
pub fn resolve_num_ctx(settings: &SessionSettings) -> i32 {
    settings.options.num_ctx.unwrap_or(DEFAULT_NUM_CTX)
}

/// Map settings to generation options. Code mode is the most deterministic;
/// agent mode wants low temperature for reliable tool use later.
pub fn build_options(settings: &SessionSettings) -> ChatOptions {
    let mode_temp = match settings.mode {
        Mode::Agent => 0.3,
        Mode::Chat => 0.7,
    };
    let temperature =
        settings
            .options
            .temperature
            .unwrap_or(if settings.code { 0.2 } else { mode_temp });

    ChatOptions {
        temperature: Some(temperature),
        top_p: Some(settings.options.top_p.unwrap_or(0.9)),
        top_k: None,
        num_ctx: Some(resolve_num_ctx(settings)),
        num_predict: settings.options.num_predict,
    }
}

/// Build the full message list: one system message, then budget-trimmed history
/// followed by the new user turn. Reasoning (`thinking`) is never sent back to
/// the model.
pub fn build_context(input: ContextInput) -> AssembledContext {
    let system = compose_system_prompt(&input.settings);
    let num_ctx = resolve_num_ctx(&input.settings) as usize;

    let mut convo: Vec<ChatMessage> = input
        .history
        .into_iter()
        .filter(|m| !m.content.trim().is_empty())
        .map(|mut m| {
            m.thinking = None;
            m
        })
        .collect();
    convo.push(ChatMessage {
        role: "user".to_string(),
        content: input.user_message,
        thinking: None,
    });

    // ponytail: reserve a quarter of the window for the reply. Tighten once
    // num_predict is set per session.
    let output_reserve = num_ctx / 4;
    let budget = num_ctx
        .saturating_sub(output_reserve)
        .saturating_sub(estimate_tokens(&system));

    let mut messages = Vec::with_capacity(convo.len() + 1);
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: system,
        thinking: None,
    });
    messages.extend(trim_to_budget(convo, budget));

    AssembledContext {
        messages,
        options: build_options(&input.settings),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            thinking: None,
        }
    }

    fn settings() -> SessionSettings {
        SessionSettings::default()
    }

    #[test]
    fn system_message_is_first_and_only_once() {
        let ctx = build_context(ContextInput {
            settings: settings(),
            history: vec![msg("user", "hi"), msg("assistant", "hello")],
            user_message: "how are you".into(),
        });
        assert_eq!(ctx.messages[0].role, "system");
        assert!(ctx.messages[0].content.contains("Solyn"));
        assert_eq!(
            ctx.messages.iter().filter(|m| m.role == "system").count(),
            1
        );
        assert_eq!(ctx.messages.last().unwrap().content, "how are you");
    }

    #[test]
    fn code_mode_is_colder_than_chat() {
        let chat = build_options(&settings()).temperature.unwrap();
        let mut s = settings();
        s.code = true;
        let code = build_options(&s).temperature.unwrap();
        assert!(code < chat);
    }

    #[test]
    fn code_mode_adds_overlay() {
        let mut s = settings();
        assert!(!compose_system_prompt(&s).contains("coding mode"));
        s.code = true;
        assert!(compose_system_prompt(&s).contains("coding mode"));
    }

    #[test]
    fn agent_mode_adds_overlay() {
        let mut s = settings();
        assert!(!compose_system_prompt(&s).contains("agent mode"));
        s.mode = Mode::Agent;
        assert!(compose_system_prompt(&s).contains("agent mode"));
    }

    #[test]
    fn default_num_ctx_is_8192_and_overridable() {
        assert_eq!(resolve_num_ctx(&settings()), DEFAULT_NUM_CTX);
        let mut s = settings();
        s.options.num_ctx = Some(32768);
        assert_eq!(resolve_num_ctx(&s), 32768);
    }

    #[test]
    fn thinking_is_stripped_from_outgoing_history() {
        let mut prior = msg("assistant", "previous");
        prior.thinking = Some("secret reasoning".into());
        let ctx = build_context(ContextInput {
            settings: settings(),
            history: vec![prior],
            user_message: "next".into(),
        });
        assert!(ctx.messages.iter().all(|m| m.thinking.is_none()));
    }

    #[test]
    fn trim_drops_oldest_pairs_and_keeps_the_newest_turn() {
        let history: Vec<ChatMessage> = (0..50)
            .map(|i| {
                msg(
                    if i % 2 == 0 { "user" } else { "assistant" },
                    &"x".repeat(4000),
                )
            })
            .collect();
        let ctx = build_context(ContextInput {
            settings: settings(),
            history,
            user_message: "final".into(),
        });
        // Trimming happened, system survived, and the current turn is intact.
        assert!(ctx.messages.len() < 52);
        assert_eq!(ctx.messages[0].role, "system");
        assert_eq!(ctx.messages.last().unwrap().content, "final");
        // Every conversation turn (everything after the system message) starts
        // with a user message.
        assert_eq!(ctx.messages[1].role, "user");
    }
}
