use super::last_assistant_text;
use crate::parse::parse_event;
use crate::test_env::with_clean_env;
use crate::{CallerKind, HookEvent};
use serde_json::json;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn fixture(name: &str) -> serde_json::Value {
    let raw = std::fs::read(fixture_path(name)).expect("fixture file should be readable");
    serde_json::from_slice(&raw).expect("fixture should be JSON")
}

/// Parse with every caller-detection env var unset, so the payload shape decides.
fn parse(payload: &serde_json::Value) -> HookEvent {
    with_clean_env(|| parse_event(payload.to_string().as_bytes()).expect("parse failed"))
}

#[test]
fn claude_code_stop_uses_last_assistant_message() {
    let evt = parse(&fixture("claude-code-stop.json"));
    assert_eq!(evt.caller, CallerKind::ClaudeCode);
    assert_eq!(evt.event.to_string(), "turn:stop");
    assert_eq!(evt.reply.as_deref(), Some("Done. The tests pass."));
    assert_eq!(
        evt.transcript_path.as_deref(),
        Some("/tmp/transcript.jsonl")
    );
}

#[test]
fn claude_code_stop_without_message_reads_transcript() {
    let mut payload = fixture("claude-code-stop-without-reply.json");
    let transcript = fixture_path("claude-code-transcript.jsonl");
    payload["transcript_path"] = json!(transcript.to_str().unwrap());
    let evt = parse(&payload);
    assert_eq!(evt.event.to_string(), "turn:stop");
    assert_eq!(
        evt.reply.as_deref(),
        Some("All tests pass.\nNothing else to do.")
    );
    assert_eq!(evt.transcript_path.as_deref(), transcript.to_str());
}

#[test]
fn claude_code_stop_with_missing_transcript_has_no_reply() {
    let evt = parse(&fixture("claude-code-stop-without-reply.json"));
    assert_eq!(evt.event.to_string(), "turn:stop");
    assert!(evt.reply.is_none());
}

#[test]
fn claude_code_session_end_stays_session_stop() {
    let evt = parse(&json!({
        "hook_event_name": "SessionEnd", "session_id": "s1", "reason": "exit",
        "transcript_path": "/t", "last_assistant_message": "stray"
    }));
    assert_eq!(evt.event.to_string(), "session:stop");
    assert!(evt.reply.is_none());
    assert_eq!(evt.transcript_path.as_deref(), Some("/t"));
}

#[test]
fn codex_stop_uses_last_assistant_message() {
    let evt = parse(&json!({
        "hook_event_name": "Stop", "session_id": "s1", "turn_id": "t1",
        "stop_hook_active": false, "last_assistant_message": "fixed"
    }));
    assert_eq!(evt.caller, CallerKind::Codex);
    assert_eq!(evt.event.to_string(), "turn:stop");
    assert_eq!(evt.reply.as_deref(), Some("fixed"));
}

#[test]
fn gemini_cli_after_agent_without_response_has_no_reply() {
    // Only Claude Code-format payloads fall back to the transcript.
    let mut payload = fixture("claude-code-stop-without-reply.json");
    payload["hook_event_name"] = json!("AfterAgent");
    payload["transcript_path"] = json!(fixture_path("claude-code-transcript.jsonl"));
    let evt = parse(&payload);
    assert_eq!(evt.caller, CallerKind::GeminiCli);
    assert_eq!(evt.event.to_string(), "turn:stop");
    assert!(evt.reply.is_none());
}

#[test]
fn gemini_cli_after_agent_uses_prompt_response() {
    let evt = parse(&json!({
        "hook_event_name": "AfterAgent", "session_id": "s1", "transcript_path": "/t",
        "prompt": "fix it", "prompt_response": "fixed", "stop_hook_active": false
    }));
    assert_eq!(evt.caller, CallerKind::GeminiCli);
    assert_eq!(evt.event.to_string(), "turn:stop");
    assert_eq!(evt.reply.as_deref(), Some("fixed"));
}

#[test]
fn hermes_post_llm_call_uses_assistant_response() {
    let evt = parse(&json!({
        "hook_event_name": "post_llm_call", "tool_name": null, "tool_input": null,
        "session_id": "s1", "cwd": "/w",
        "extra": {"user_message": "fix it", "assistant_response": "fixed"}
    }));
    assert_eq!(evt.caller, CallerKind::Hermes);
    assert_eq!(evt.event.to_string(), "turn:stop");
    assert_eq!(evt.reply.as_deref(), Some("fixed"));
}

#[test]
fn transcript_without_assistant_text_has_no_reply() {
    let jsonl = concat!(
        r#"{"type":"user","message":{"content":"hi"}}"#,
        "\n",
        r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash"}]}}"#,
        "\nnot json\n",
    );
    assert_eq!(last_assistant_text(jsonl), None);
}
