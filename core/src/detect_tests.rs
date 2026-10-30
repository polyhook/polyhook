use super::detect_caller;
use crate::CallerKind;

use crate::test_env::with_clean_env;

#[test]
fn claude_code_version_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("CLAUDE_CODE_VERSION", Some("1.0.0"), || {
            assert_eq!(detect_caller(&val), CallerKind::ClaudeCode);
        });
    });
}

#[test]
fn cursor_session_id_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("CURSOR_SESSION_ID", Some("cursor-sess-abc"), || {
            assert_eq!(detect_caller(&val), CallerKind::Cursor);
        });
    });
}

#[test]
fn windsurf_session_id_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("WINDSURF_SESSION_ID", Some("ws-sess-xyz"), || {
            assert_eq!(detect_caller(&val), CallerKind::Windsurf);
        });
    });
}

#[test]
fn cline_session_id_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("CLINE_SESSION_ID", Some("cline-sess-999"), || {
            assert_eq!(detect_caller(&val), CallerKind::Cline);
        });
    });
}

#[test]
fn amp_session_id_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("AMP_SESSION_ID", Some("amp-sess-000"), || {
            assert_eq!(detect_caller(&val), CallerKind::Amp);
        });
    });
}

#[test]
fn polyhook_caller_garbage_falls_through_to_heuristics() {
    let val = serde_json::json!({"tool_name": "Bash", "tool_input": {}, "session_id": "s1"});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("garbage_value_xyz"), || {
            assert_eq!(detect_caller(&val), CallerKind::ClaudeCode);
        });
    });
}

#[test]
fn polyhook_caller_garbage_with_no_heuristic_match_returns_unknown() {
    let val = serde_json::json!({"some_random_key": "some_value"});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("not_a_known_caller"), || {
            assert_eq!(detect_caller(&val), CallerKind::Unknown);
        });
    });
}

#[test]
fn unknown_json_shape_returns_unknown() {
    let val = serde_json::json!({"foo": "bar"});
    with_clean_env(|| {
        assert_eq!(detect_caller(&val), CallerKind::Unknown);
    });
}

#[test]
fn empty_object_returns_unknown() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        assert_eq!(detect_caller(&val), CallerKind::Unknown);
    });
}

#[test]
fn non_object_json_returns_unknown() {
    let val = serde_json::json!(["tool_name", "tool_input"]);
    with_clean_env(|| {
        assert_eq!(detect_caller(&val), CallerKind::Unknown);
    });
}

#[test]
fn polyhook_caller_claudecode_alias_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("claudecode"), || {
            assert_eq!(detect_caller(&val), CallerKind::ClaudeCode);
        });
    });
}

#[test]
fn gemini_project_dir_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("GEMINI_PROJECT_DIR", Some("/home/user/project"), || {
            assert_eq!(detect_caller(&val), CallerKind::GeminiCli);
        });
    });
}

#[test]
fn polyhook_caller_gemini_cli_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("gemini-cli"), || {
            assert_eq!(detect_caller(&val), CallerKind::GeminiCli);
        });
    });
}

#[test]
fn polyhook_caller_hermes_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("hermes"), || {
            assert_eq!(detect_caller(&val), CallerKind::Hermes);
        });
    });
}

#[test]
fn polyhook_caller_hermes_agent_alias_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("hermes-agent"), || {
            assert_eq!(detect_caller(&val), CallerKind::Hermes);
        });
    });
}

#[test]
fn polyhook_caller_pi_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("pi"), || {
            assert_eq!(detect_caller(&val), CallerKind::Pi);
        });
    });
}

#[test]
fn gemini_cli_before_tool_heuristic() {
    let val = serde_json::json!({
        "hook_event_name": "BeforeTool",
        "tool_name": "run_shell_command",
        "tool_input": {"command": "ls"},
        "session_id": "s1"
    });
    with_clean_env(|| {
        assert_eq!(detect_caller(&val), CallerKind::GeminiCli);
    });
}

#[test]
fn gemini_cli_session_start_heuristic() {
    let val = serde_json::json!({
        "hook_event_name": "SessionStart",
        "session_id": "s1",
        "source": "cli"
    });
    with_clean_env(|| {
        assert_eq!(detect_caller(&val), CallerKind::GeminiCli);
    });
}

#[test]
fn gemini_cli_notification_does_not_match_heuristic() {
    // "Notification" is ambiguous (shared with Claude Code / Cursor), so the
    // heuristic deliberately does not match it; env var detection is used instead.
    let val = serde_json::json!({"hook_event_name": "Notification", "session_id": "s1"});
    with_clean_env(|| {
        assert_eq!(detect_caller(&val), CallerKind::Unknown);
    });
}

#[test]
fn hermes_pre_tool_call_heuristic_beats_claude_code_shape() {
    let val = serde_json::json!({
        "hook_event_name": "pre_tool_call",
        "tool_name": "terminal",
        "tool_input": {"command": "ls"},
        "session_id": "s1"
    });
    with_clean_env(|| {
        assert_eq!(detect_caller(&val), CallerKind::Hermes);
    });
}

#[test]
fn hermes_session_start_heuristic() {
    let val = serde_json::json!({
        "hook_event_name": "on_session_start",
        "session_id": "s1",
        "cwd": "/tmp/project"
    });
    with_clean_env(|| {
        assert_eq!(detect_caller(&val), CallerKind::Hermes);
    });
}

#[test]
fn claude_code_session_end_with_project_dir_env_detected() {
    let val = serde_json::json!({
        "hook_event_name": "SessionEnd",
        "session_id": "s1",
        "reason": "other"
    });
    with_clean_env(|| {
        temp_env::with_var("CLAUDE_PROJECT_DIR", Some("/proj"), || {
            assert_eq!(detect_caller(&val), CallerKind::ClaudeCode);
        });
    });
}

#[test]
fn gemini_cli_claude_project_dir_alias_stays_gemini() {
    // Gemini CLI sets CLAUDE_PROJECT_DIR as an alias alongside GEMINI_PROJECT_DIR.
    let val = serde_json::json!({"hook_event_name": "SessionEnd", "session_id": "s1"});
    with_clean_env(|| {
        temp_env::with_vars(
            [
                ("GEMINI_PROJECT_DIR", Some("/proj")),
                ("CLAUDE_PROJECT_DIR", Some("/proj")),
            ],
            || assert_eq!(detect_caller(&val), CallerKind::GeminiCli),
        );
    });
}

#[test]
fn claude_project_dir_alone_does_not_claim_non_shared_payloads() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("CLAUDE_PROJECT_DIR", Some("/proj"), || {
            assert_eq!(detect_caller(&val), CallerKind::Unknown);
        });
    });
}

#[test]
fn codex_turn_id_beats_claude_code_event_names() {
    with_clean_env(|| {
        temp_env::with_var("CLAUDE_PROJECT_DIR", Some("/proj"), || {
            for name in ["PreToolUse", "Stop", "UserPromptSubmit", "SessionStart"] {
                let val = serde_json::json!({
                    "hook_event_name": name, "session_id": "s1", "turn_id": "t1"
                });
                assert_eq!(detect_caller(&val), CallerKind::Codex, "{name}");
            }
        });
    });
}

#[test]
fn claude_code_lifecycle_event_names_detected_without_tool_fields() {
    with_clean_env(|| {
        for name in [
            "PreToolUse",
            "PostToolUse",
            "Stop",
            "SubagentStop",
            "UserPromptSubmit",
            "PreCompact",
            "PermissionRequest",
        ] {
            let val = serde_json::json!({"hook_event_name": name, "session_id": "s1"});
            assert_eq!(detect_caller(&val), CallerKind::ClaudeCode, "{name}");
        }
    });
}

#[test]
fn claude_code_session_start_with_project_dir_env_beats_gemini_heuristic() {
    let val = serde_json::json!({
        "hook_event_name": "SessionStart",
        "session_id": "s1",
        "source": "startup"
    });
    with_clean_env(|| {
        temp_env::with_var("CLAUDE_PROJECT_DIR", Some("/proj"), || {
            assert_eq!(detect_caller(&val), CallerKind::ClaudeCode);
        });
    });
}

// ---------------------------------------------------------------------------
// Environment overlay (set by the WASM SDKs, which have no real environment)
// ---------------------------------------------------------------------------

use super::{env_var, set_env_overlay};
use std::collections::HashMap;

fn overlay(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

// Run `f` with an overlay installed, always clearing it afterwards so the
// thread-local cannot leak into another test on the same thread.
fn with_overlay<R>(pairs: &[(&str, &str)], f: impl FnOnce() -> R) -> R {
    set_env_overlay(Some(overlay(pairs)));
    let out = f();
    set_env_overlay(None);
    out
}

#[test]
fn overlay_supplies_polyhook_caller_when_process_env_is_empty() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        with_overlay(&[("POLYHOOK_CALLER", "windsurf")], || {
            assert_eq!(detect_caller(&val), CallerKind::Windsurf);
        });
    });
}

#[test]
fn overlay_polyhook_caller_beats_json_shape_heuristic() {
    // Shape says Cursor; the explicit override must win (priority 1 over 3).
    let val = serde_json::json!({"type": "BeforeToolCall", "toolCall": {"name": "shell"}});
    with_clean_env(|| {
        assert_eq!(detect_caller(&val), CallerKind::Cursor);
        with_overlay(&[("POLYHOOK_CALLER", "windsurf")], || {
            assert_eq!(detect_caller(&val), CallerKind::Windsurf);
        });
    });
}

#[test]
fn overlay_supplies_agent_specific_env_vars() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        for (var, want) in [
            ("CLAUDE_CODE_VERSION", CallerKind::ClaudeCode),
            ("CURSOR_SESSION_ID", CallerKind::Cursor),
            ("WINDSURF_SESSION_ID", CallerKind::Windsurf),
            ("CLINE_SESSION_ID", CallerKind::Cline),
            ("AMP_SESSION_ID", CallerKind::Amp),
            ("GEMINI_PROJECT_DIR", CallerKind::GeminiCli),
        ] {
            with_overlay(&[(var, "x")], || {
                assert_eq!(detect_caller(&val), want, "{var}");
            });
        }
    });
}

#[test]
fn overlay_without_the_key_does_not_fall_back_to_process_env() {
    // A host that pushed its env in has already told us everything it has, so
    // a miss must stay a miss rather than reading the real environment.
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("amp"), || {
            assert_eq!(env_var("POLYHOOK_CALLER").as_deref(), Some("amp"));
            with_overlay(&[("UNRELATED", "1")], || {
                assert_eq!(env_var("POLYHOOK_CALLER"), None);
            });
            // Cleared again: the real environment is visible once more.
            assert_eq!(env_var("POLYHOOK_CALLER").as_deref(), Some("amp"));
        });
    });
}

#[test]
fn no_overlay_reads_the_process_env() {
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("codex"), || {
            assert_eq!(env_var("POLYHOOK_CALLER").as_deref(), Some("codex"));
        });
    });
}
