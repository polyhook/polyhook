//! Tests for `detect_caller` — agent-specific env vars and edge cases not
//! covered by the sdk-rust integration tests.

use polyhook_core::detect::detect_caller;
use polyhook_core::types::CallerKind;

/// All env vars that detect_caller inspects. We clear them all before each
/// test to prevent cross-test interference.
const AGENT_ENV_VARS: &[&str] = &[
    "POLYHOOK_CALLER",
    "CLAUDE_CODE_VERSION",
    "CURSOR_SESSION_ID",
    "WINDSURF_SESSION_ID",
    "CLINE_SESSION_ID",
    "AMP_SESSION_ID",
];

fn with_clean_env<F: FnOnce()>(f: F) {
    let vars: Vec<(&str, Option<&str>)> = AGENT_ENV_VARS.iter().map(|k| (*k, None)).collect();
    temp_env::with_vars(vars, f);
}

// ---------------------------------------------------------------------------
// Agent-specific env vars (section 2 of detect.rs)
// ---------------------------------------------------------------------------

#[test]
fn claude_code_version_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("CLAUDE_CODE_VERSION", Some("1.0.0"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::ClaudeCode);
        });
    });
}

#[test]
fn cursor_session_id_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("CURSOR_SESSION_ID", Some("cursor-sess-abc"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::Cursor);
        });
    });
}

#[test]
fn windsurf_session_id_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("WINDSURF_SESSION_ID", Some("ws-sess-xyz"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::Windsurf);
        });
    });
}

#[test]
fn cline_session_id_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("CLINE_SESSION_ID", Some("cline-sess-999"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::Cline);
        });
    });
}

#[test]
fn amp_session_id_env_var_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("AMP_SESSION_ID", Some("amp-sess-000"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::Amp);
        });
    });
}

// ---------------------------------------------------------------------------
// POLYHOOK_CALLER with an unrecognized value falls through to heuristics
// ---------------------------------------------------------------------------

#[test]
fn polyhook_caller_garbage_falls_through_to_heuristics_claude_code() {
    // JSON matches ClaudeCode heuristic (has "tool_name" and "tool_input").
    let val = serde_json::json!({
        "tool_name": "Bash",
        "tool_input": {"command": "ls"},
        "session_id": "s1"
    });
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("garbage_value_xyz"), || {
            // The unrecognized value falls through; heuristics detect ClaudeCode.
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::ClaudeCode);
        });
    });
}

#[test]
fn polyhook_caller_garbage_falls_through_to_heuristics_cursor() {
    // JSON matches Cursor heuristic (has "type" and "toolCall").
    let val = serde_json::json!({
        "type": "BeforeToolCall",
        "toolCall": {"name": "run_terminal_cmd", "args": {}},
        "sessionId": "s2"
    });
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("__unknown__"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::Cursor);
        });
    });
}

#[test]
fn polyhook_caller_garbage_with_no_heuristic_match_returns_unknown() {
    // JSON has no keys that match any heuristic.
    let val = serde_json::json!({"some_random_key": "some_value"});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("not_a_known_caller"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::Unknown);
        });
    });
}

// ---------------------------------------------------------------------------
// Unknown branch: JSON has none of the heuristic keys
// ---------------------------------------------------------------------------

#[test]
fn unknown_json_shape_returns_unknown() {
    let val = serde_json::json!({"foo": "bar", "baz": 42});
    with_clean_env(|| {
        let caller = detect_caller(&val);
        assert_eq!(caller, CallerKind::Unknown);
    });
}

#[test]
fn empty_object_returns_unknown() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        let caller = detect_caller(&val);
        assert_eq!(caller, CallerKind::Unknown);
    });
}

#[test]
fn non_object_json_value_returns_unknown() {
    // A JSON array — not an object, so heuristics can't inspect keys.
    let val = serde_json::json!(["tool_name", "tool_input"]);
    with_clean_env(|| {
        let caller = detect_caller(&val);
        assert_eq!(caller, CallerKind::Unknown);
    });
}

// ---------------------------------------------------------------------------
// POLYHOOK_CALLER alternate spellings (already in sdk-rust tests for some, but
// the "claudecode" alias is only tested in detect.rs itself, not via a unit test)
// ---------------------------------------------------------------------------

#[test]
fn polyhook_caller_claudecode_alias_detected() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("claudecode"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::ClaudeCode);
        });
    });
}
