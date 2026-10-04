use crate::types::CallerKind;

/// Detect which agent is calling the hook.
///
/// Priority:
/// 1. `POLYHOOK_CALLER` env var (explicit override)
/// 2. Agent-specific env vars
/// 3. Heuristics on the raw stdin JSON shape
/// 4. `Unknown`
pub fn detect_caller(stdin: &serde_json::Value) -> CallerKind {
    // 1. Explicit override via env var
    if let Ok(val) = std::env::var("POLYHOOK_CALLER") {
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
    if std::env::var("CLAUDE_CODE_VERSION").is_ok() {
        return CallerKind::ClaudeCode;
    }
    if std::env::var("CURSOR_SESSION_ID").is_ok() {
        return CallerKind::Cursor;
    }
    if std::env::var("WINDSURF_SESSION_ID").is_ok() {
        return CallerKind::Windsurf;
    }
    if std::env::var("CLINE_SESSION_ID").is_ok() {
        return CallerKind::Cline;
    }
    if std::env::var("AMP_SESSION_ID").is_ok() {
        return CallerKind::Amp;
    }
    if std::env::var("GEMINI_PROJECT_DIR").is_ok() {
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
                if std::env::var("CLAUDE_PROJECT_DIR").is_err() {
                    return CallerKind::GeminiCli;
                }
                if !has("turn_id") {
                    return CallerKind::ClaudeCode;
                }
            }
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

        // Windsurf Cascade's real hook contract: every payload names its
        // event in `agent_action_name` and carries a `tool_info` object
        // (https://docs.windsurf.com/windsurf/cascade/hooks).
        if has("agent_action_name") {
            return CallerKind::Windsurf;
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
