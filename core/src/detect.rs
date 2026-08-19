use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::CallerKind;

// Environment supplied by a host that cannot expose a real process env to us
// (the WASM ABI: `wasm32-unknown-unknown` has no `std::env`). Consulted before
// `std::env::var` so the same detection logic serves native and WASM callers.
thread_local! {
    static HOST_ENV: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Replace the host-supplied environment used by [`detect_caller`].
///
/// SDK shims running the WASM build call this (via the `set_env` export) with
/// the host process environment before `parse`, because `std::env::var` is
/// always empty inside `wasm32-unknown-unknown`.
pub fn set_host_env(env: HashMap<String, String>) {
    HOST_ENV.with(|e| *e.borrow_mut() = env);
}

/// Look up an environment variable: host-supplied env first, then the real
/// process environment.
fn env_var(name: &str) -> Option<String> {
    HOST_ENV
        .with(|e| e.borrow().get(name).cloned())
        .or_else(|| std::env::var(name).ok())
}

/// Detect which agent is calling the hook.
///
/// Priority:
/// 1. `POLYHOOK_CALLER` env var (explicit override)
/// 2. Agent-specific env vars
/// 3. Heuristics on the raw stdin JSON shape
/// 4. `Unknown`
pub fn detect_caller(stdin: &serde_json::Value) -> CallerKind {
    // 1. Explicit override via env var
    if let Some(val) = env_var("POLYHOOK_CALLER") {
        match val.to_lowercase().as_str() {
            "claude-code" | "claudecode" => return CallerKind::ClaudeCode,
            "cursor" => return CallerKind::Cursor,
            "windsurf" => return CallerKind::Windsurf,
            "cline" => return CallerKind::Cline,
            "amp" => return CallerKind::Amp,
            "gemini-cli" | "geminicli" => return CallerKind::GeminiCli,
            "hermes" | "hermes-agent" | "hermesagent" => return CallerKind::Hermes,
            "pi" => return CallerKind::Pi,
            _ => {}
        }
    }

    // 2. Agent-specific env vars
    if env_var("CLAUDE_CODE_VERSION").is_some() {
        return CallerKind::ClaudeCode;
    }
    if env_var("CURSOR_SESSION_ID").is_some() {
        return CallerKind::Cursor;
    }
    if env_var("WINDSURF_SESSION_ID").is_some() {
        return CallerKind::Windsurf;
    }
    if env_var("CLINE_SESSION_ID").is_some() {
        return CallerKind::Cline;
    }
    if env_var("AMP_SESSION_ID").is_some() {
        return CallerKind::Amp;
    }
    if env_var("GEMINI_PROJECT_DIR").is_some() {
        return CallerKind::GeminiCli;
    }

    // 3. JSON shape heuristics
    if let Some(obj) = stdin.as_object() {
        let has = |key: &str| obj.contains_key(key);
        let str_val = |key: &str| obj.get(key).and_then(|v| v.as_str()).unwrap_or("");

        // Gemini CLI / Hermes: hook_event_name with caller-specific values.
        // Checked before the Claude Code heuristic because all three send
        // tool_name + tool_input for tool events.
        match str_val("hook_event_name") {
            "BeforeTool"
            | "AfterTool"
            | "BeforeAgent"
            | "AfterAgent"
            | "BeforeModel"
            | "AfterModel"
            | "BeforeToolSelection"
            | "PreCompress"
            | "SessionStart"
            | "SessionEnd" => return CallerKind::GeminiCli,
            "pre_tool_call"
            | "post_tool_call"
            | "pre_llm_call"
            | "on_session_start"
            | "on_session_end"
            | "on_session_finalize"
            | "subagent_stop" => {
                return CallerKind::Hermes;
            }
            _ => {}
        }

        if has("tool_name") && has("tool_input") {
            return CallerKind::ClaudeCode;
        }
        if has("type") && has("toolCall") {
            return CallerKind::Cursor;
        }
        if has("event") && has("parameters") {
            return CallerKind::Windsurf;
        }
        // Cline uses toolName (not toolCall)
        if has("type") && has("toolName") && !has("toolCall") {
            return CallerKind::Cline;
        }
        if has("kind") {
            return CallerKind::Amp;
        }
    }

    CallerKind::Unknown
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "detect_tests.rs"]
mod tests;
