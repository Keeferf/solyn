//! System prompt composition.
//!
//! Phase 0 only composes from mode + the code toggle. The web toggle is persisted
//! but has no overlay yet — it becomes a real tool in Phase 3, and a prompt that
//! promises search before then would be a lie.

use super::{Mode, SessionSettings};

pub const BASE_PROMPT: &str =
    "You are Solyn, a private assistant running entirely on the user's machine. \
Answer directly and concisely. Use Markdown for code, lists, and tables. \
If you are unsure or lack information, say so rather than guessing.";

pub const AGENT_OVERLAY: &str = "You are in agent mode. Work through the problem end to end: \
state a short plan, then carry it out step by step. Be methodical, and check your work before concluding.";

pub const CODE_OVERLAY: &str =
    "You are in coding mode. Reply with code first and keep prose to a minimum. \
Do not add pleasantries, restate the request, or pad the answer. \
Prefer complete, runnable snippets and call out required imports. \
If the request is ambiguous, make the most reasonable assumption and state it in one line.";

/// Compose the single system message placed at the head of every request.
pub fn compose_system_prompt(settings: &SessionSettings) -> String {
    let mut parts: Vec<&str> = vec![BASE_PROMPT];
    if settings.mode == Mode::Agent {
        parts.push(AGENT_OVERLAY);
    }
    if settings.code {
        parts.push(CODE_OVERLAY);
    }
    parts.join("\n\n")
}
