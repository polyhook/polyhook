use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::CallerKind;

thread_local! {
    /// Host environment supplied by a caller that cannot expose the real
    /// process environment to this code. `wasm32-unknown-unknown` has no
    /// environment at all — `std::env::var` there always returns
    /// `Err(NotPresent)` — so the WASM-backed SDKs push the host env in via
    /// `wasm::set_env` and this overlay is consulted first.
    static ENV_OVERLAY: RefCell<Option<HashMap<String, String>>> = const { RefCell::new(None) };
}

/// Replace the environment overlay consulted by [`env_var`].
///
/// Passing `None` clears it, restoring plain `std::env::var` lookups.
pub fn set_env_overlay(vars: Option<HashMap<String, String>>) {
    ENV_OVERLAY.with(|o| {
        *o.borrow_mut() = vars;
    });
}

/// Read an environment variable, preferring the overlay set by
/// [`set_env_overlay`] and falling back to the real process environment.
///
/// An overlay that is present but lacks `name` does not fall through: a host
/// that pushed its environment in has already told us everything it has, so
/// falling back would only ever read the (empty) WASM environment.
pub fn env_var(name: &str) -> Option<String> {
    let overlaid = ENV_OVERLAY.with(|o| {
        o.borrow()
            .as_ref()
            .map(|vars| vars.get(name).map(String::to_owned))
    });
    match overlaid {
        Some(hit) => hit,
        None => std::env::var(name).ok(),
    }
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
            "codex" => return CallerKind::Codex,
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
            | "PreCompress" => return CallerKind::GeminiCli,
            // Shared with Claude Code. Claude Code sets CLAUDE_PROJECT_DIR for
            // every hook (https://code.claude.com/docs/en/hooks); Gemini CLI
            // sets it too, as an alias, but also sets GEMINI_PROJECT_DIR,
            // which step 2 already matched. Codex payloads carry `turn_id`.
            "SessionStart" | "SessionEnd" => {
                if env_var("CLAUDE_PROJECT_DIR").is_none() {
                    return CallerKind::GeminiCli;
                }
                if !has("turn_id") {
                    return CallerKind::ClaudeCode;
                }
            }
            "pre_tool_call"
            | "post_tool_call"
            | "pre_llm_call"
            | "post_llm_call"
            | "on_session_start"
            | "on_session_end"
            | "on_session_finalize"
            | "subagent_stop" => {
                return CallerKind::Hermes;
            }
            _ => {}
        }

        // Cline's file hooks name the event in `hookName` and tag every
        // payload with `clineVersion`.
        if has("hookName") && has("clineVersion") {
            return CallerKind::Cline;
        }

        // Codex speaks Claude Code's hook format but adds a per-turn
        // `turn_id` to every payload, which Claude Code never sends.
        if has("hook_event_name") && has("turn_id") {
            return CallerKind::Codex;
        }

        // Claude Code-only names, checked after Codex (which reuses them).
        // Lifecycle events (Stop, SubagentStop, …) carry no tool_name/tool_input,
        // and prompt events carry the prompt, so the shape check below misses them.
        if matches!(
            str_val("hook_event_name"),
            "PreToolUse"
                | "PostToolUse"
                | "Stop"
                | "SubagentStop"
                | "UserPromptSubmit"
                | "PreCompact"
                | "PermissionRequest"
        ) {
            return CallerKind::ClaudeCode;
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
