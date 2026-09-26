//! Token estimation and context-window trimming.
//!
//! Token counting uses a character heuristic rather than a tokenizer. This keeps
//! Phase 0 dependency-free; if truncation ever misbehaves, swap `estimate_tokens`
//! for a real tokenizer (Ollama exposes no tokenize endpoint).

use crate::core::ollama::chat::ChatMessage;

/// ponytail: ~4 chars/token is an estimate, not a tokenizer.
pub const CHARS_PER_TOKEN: usize = 4;

/// Fallback context window when neither the session nor the model provides one.
/// Doubles Ollama's ~4k default; Phase 6 replaces this with a per-model/hardware resolver.
pub const DEFAULT_NUM_CTX: i32 = 8192;

/// Approximate per-message chat-template overhead, in tokens.
const MESSAGE_OVERHEAD_TOKENS: usize = 4;

pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(CHARS_PER_TOKEN)
}

pub fn estimate_messages_tokens(messages: &[ChatMessage]) -> usize {
    messages
        .iter()
        .map(|m| estimate_tokens(&m.content) + MESSAGE_OVERHEAD_TOKENS)
        .sum()
}

/// Drop the oldest whole turns until the conversation fits the budget.
///
/// The newest message (the current user turn) is never dropped, and turns are
/// removed in pairs so a user/assistant exchange is never split.
pub fn trim_to_budget(mut messages: Vec<ChatMessage>, budget_tokens: usize) -> Vec<ChatMessage> {
    while messages.len() > 1 && estimate_messages_tokens(&messages) > budget_tokens {
        let drop = if messages.len() >= 3 { 2 } else { 1 };
        messages.drain(0..drop);
    }
    messages
}
