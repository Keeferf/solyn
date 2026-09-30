//! Context assembly: the single place that turns a session's settings, prior
//! history, and the new user message into the exact payload sent to Ollama.
//!
//! Every later feature (code mode, attachments, tools, RAG, context tuning)
//! plugs into `ContextInput` rather than growing a new pipeline. Keeping this in
//! Rust means the token budget is authoritative and the agent loop (Phase 4) can
//! re-assemble server-side without round-tripping through the webview.

mod budget;
mod prompts;

pub use budget::{
    estimate_messages_tokens, estimate_tokens, trim_to_budget, CHARS_PER_TOKEN, DEFAULT_NUM_CTX,
};
pub use prompts::compose_system_prompt;

use serde::{Deserialize, Serialize};

use crate::core::attachments::AttachmentMeta;
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
    /// Files attached for the whole session. Only paths/metadata are persisted;
    /// contents are re-read at send time.
    #[serde(default)]
    pub attachments: Vec<AttachmentMeta>,
}

/// A delimited chunk of external content (an attached file now, RAG later).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBlock {
    pub label: String,
    pub content: String,
}

pub struct ContextInput {
    pub settings: SessionSettings,
    pub history: Vec<ChatMessage>,
    pub user_message: String,
    pub context_blocks: Vec<ContextBlock>,
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
    let persona = compose_system_prompt(&input.settings);
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
    let window = num_ctx.saturating_sub(output_reserve);

    // Attachments live on the system message so trimming can never drop them.
    // They get at most half the free window, so a large file can't starve the
    // conversation.
    let files_budget = window.saturating_sub(estimate_tokens(&persona)) / 2;
    let system = match render_session_files(&input.context_blocks, files_budget) {
        Some(files) => format!("{persona}\n\n{files}"),
        None => persona,
    };

    let budget = window.saturating_sub(estimate_tokens(&system));

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

/// Render attachment contents into one framed section, clipped to `budget_tokens`.
/// Returns `None` when there is nothing to add.
fn render_session_files(blocks: &[ContextBlock], budget_tokens: usize) -> Option<String> {
    if blocks.is_empty() || budget_tokens < 32 {
        return None;
    }

    let max_chars = budget_tokens.saturating_mul(CHARS_PER_TOKEN);
    let mut out = String::from("Session files attached by the user:");
    let mut used = 0usize;

    for block in blocks {
        let header = format!("\n\n--- BEGIN FILE: {} ---\n", block.label);
        let footer = format!("\n--- END FILE: {} ---", block.label);
        let fixed = header.chars().count() + footer.chars().count();
        let body = block.content.chars().count();

        if used + fixed + body > max_chars {
            let remaining = max_chars.saturating_sub(used + fixed);
            if remaining == 0 {
                break;
            }
            out.push_str(&header);
            out.extend(block.content.chars().take(remaining));
            if !block.content.ends_with("[... truncated ...]") {
                out.push_str("\n[... truncated ...]");
            }
            out.push_str(&footer);
            break;
        }

        out.push_str(&header);
        out.push_str(&block.content);
        out.push_str(&footer);
        used += fixed + body;
    }

    Some(out)
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
            context_blocks: Vec::new(),
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
            context_blocks: Vec::new(),
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
            context_blocks: Vec::new(),
        });
        // Trimming happened, system survived, and the current turn is intact.
        assert!(ctx.messages.len() < 52);
        assert_eq!(ctx.messages[0].role, "system");
        assert_eq!(ctx.messages.last().unwrap().content, "final");
        // Every conversation turn (everything after the system message) starts
        // with a user message.
        assert_eq!(ctx.messages[1].role, "user");
    }

    #[test]
    fn session_files_are_added_to_the_system_message() {
        let ctx = build_context(ContextInput {
            settings: settings(),
            history: Vec::new(),
            user_message: "summarize this".into(),
            context_blocks: vec![ContextBlock {
                label: "notes.txt".into(),
                content: "the answer is 42".into(),
            }],
        });
        assert_eq!(ctx.messages[0].role, "system");
        assert!(ctx.messages[0].content.contains("BEGIN FILE: notes.txt"));
        assert!(ctx.messages[0].content.contains("the answer is 42"));
    }

    #[test]
    fn session_files_are_clipped_and_do_not_drop_the_newest_turn() {
        let ctx = build_context(ContextInput {
            settings: settings(),
            history: Vec::new(),
            user_message: "keep me".into(),
            context_blocks: vec![ContextBlock {
                label: "huge.txt".into(),
                content: "x".repeat(100_000),
            }],
        });
        assert_eq!(ctx.messages.last().unwrap().content, "keep me");
        assert!(ctx.messages[0].content.contains("[... truncated ...]"));
    }

    #[test]
    fn no_context_blocks_leaves_the_system_message_clean() {
        let ctx = build_context(ContextInput {
            settings: settings(),
            history: Vec::new(),
            user_message: "hi".into(),
            context_blocks: Vec::new(),
        });
        assert!(!ctx.messages[0].content.contains("Session files"));
    }
}
