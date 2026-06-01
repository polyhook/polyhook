pub mod detect;
pub mod events;
pub mod parse;
pub mod response;
pub mod tools;
pub mod types;
mod type_impls;
pub mod wasm;

pub use types::*;

use std::cell::RefCell;
use std::io::{Read, Write};

use parse::parse_event;
use response::serialize_response;

// Store the caller from the most recently parsed event so that `respond` can
// serialise the response in the correct format without the caller needing to
// thread the CallerKind through their code.
thread_local! {
    static LAST_CALLER: RefCell<CallerKind> = RefCell::new(CallerKind::Unknown);
}

/// Read a [`HookEvent`] from an arbitrary reader.
///
/// Reads until EOF, then parses the JSON payload.  The detected [`CallerKind`]
/// is stored in a thread-local so that a subsequent [`respond_to`] call can
/// serialise the response in the correct format.
pub fn read_from(r: &mut impl Read) -> Result<HookEvent, String> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf)
        .map_err(|e| format!("read error: {e}"))?;

    let event = parse_event(&buf)?;

    // Persist caller so `respond_to` / `respond` can use it.
    LAST_CALLER.with(|c| {
        *c.borrow_mut() = event.caller.clone();
    });

    Ok(event)
}

/// Write a [`HookResponse`] to an arbitrary writer in the format expected by
/// the agent that was detected during the most recent [`read_from`] call.
pub fn respond_to(w: &mut impl Write, response: &HookResponse) -> Result<(), String> {
    let caller = LAST_CALLER.with(|c| c.borrow().clone());
    let value = serialize_response(response, &caller);
    let json = serde_json::to_string(&value).map_err(|e| format!("JSON encode error: {e}"))?;

    w.write_all(json.as_bytes())
        .map_err(|e| format!("write error: {e}"))?;

    Ok(())
}

/// Read a [`HookEvent`] from standard input.
///
/// Blocks until stdin is fully closed (i.e. the invoking agent has written the
/// complete JSON payload and closed its end of the pipe).
pub fn read() -> Result<HookEvent, String> {
    read_from(&mut std::io::stdin())
}

/// Write a [`HookResponse`] to standard output in the format expected by the
/// agent that was detected during the most recent [`read`] call.
pub fn respond(response: &HookResponse) -> Result<(), String> {
    respond_to(&mut std::io::stdout(), response)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const CLAUDE_PRE_TOOL: &str = r#"{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls -la"},"session_id":"sess_test_001"}"#;
    const CURSOR_BEFORE_TOOL: &str = r#"{"type":"BeforeToolCall","toolCall":{"name":"run_terminal_cmd","args":{"command":"echo hi"}},"sessionId":"sess_test_002"}"#;

    #[test]
    fn read_from_parses_claude_code_event() {
        let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
        let event = read_from(&mut cursor).expect("read_from should succeed");
        assert_eq!(event.caller, CallerKind::ClaudeCode);
        assert_eq!(event.event.to_string(), "tool:before");
        assert_eq!(event.tool.as_deref(), Some("bash"));
        assert_eq!(event.session_id, "sess_test_001");
    }

    #[test]
    fn read_from_returns_error_on_invalid_json() {
        let mut cursor = Cursor::new(b"not valid json" as &[u8]);
        let result = read_from(&mut cursor);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("JSON parse error") || msg.contains("parse"));
    }

    #[test]
    fn respond_to_writes_json_to_writer() {
        // First set the LAST_CALLER via read_from so respond_to uses ClaudeCode format.
        let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
        let _ = read_from(&mut cursor).expect("read_from should succeed");

        let mut output: Vec<u8> = Vec::new();
        respond_to(&mut output, &HookResponse::approve()).expect("respond_to should succeed");

        let json: serde_json::Value =
            serde_json::from_slice(&output).expect("output should be valid JSON");
        // ClaudeCode approve → empty object
        assert!(json.as_object().unwrap().is_empty());
    }

    #[test]
    fn respond_to_block_uses_detected_caller() {
        // Parse a Cursor event so LAST_CALLER becomes Cursor.
        let mut cursor = Cursor::new(CURSOR_BEFORE_TOOL.as_bytes());
        let _ = read_from(&mut cursor).expect("read_from should succeed");

        let mut output: Vec<u8> = Vec::new();
        respond_to(&mut output, &HookResponse::block("stop")).expect("respond_to should succeed");

        let json: serde_json::Value =
            serde_json::from_slice(&output).expect("output should be valid JSON");
        // Cursor block → {"action": "deny", "message": "..."}
        assert_eq!(json["action"], "deny");
        assert_eq!(json["message"], "stop");
    }

    #[test]
    fn respond_to_modify_uses_detected_caller() {
        // Parse a Claude Code event.
        let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
        let _ = read_from(&mut cursor).expect("read_from should succeed");

        let new_input = serde_json::json!({"command": "echo safe"});
        let mut output: Vec<u8> = Vec::new();
        respond_to(&mut output, &HookResponse::modify(new_input.clone()))
            .expect("respond_to should succeed");

        let json: serde_json::Value =
            serde_json::from_slice(&output).expect("output should be valid JSON");
        // ClaudeCode modify → {"decision": "approve", "tool_input": {...}}
        assert_eq!(json["decision"], "approve");
        assert_eq!(json["tool_input"], new_input);
    }

    #[test]
    fn last_caller_thread_local_is_updated_by_read_from() {
        // Parse ClaudeCode event → LAST_CALLER should be ClaudeCode.
        let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
        let event = read_from(&mut cursor).expect("read_from should succeed");
        assert_eq!(event.caller, CallerKind::ClaudeCode);

        // respond_to should use ClaudeCode format.
        let mut output: Vec<u8> = Vec::new();
        respond_to(&mut output, &HookResponse::approve()).expect("respond_to should succeed");
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert!(json.as_object().unwrap().is_empty());

        // Now parse Cursor event → LAST_CALLER should switch to Cursor.
        let mut cursor2 = Cursor::new(CURSOR_BEFORE_TOOL.as_bytes());
        let event2 = read_from(&mut cursor2).expect("read_from should succeed");
        assert_eq!(event2.caller, CallerKind::Cursor);

        let mut output2: Vec<u8> = Vec::new();
        respond_to(&mut output2, &HookResponse::approve()).expect("respond_to should succeed");
        let json2: serde_json::Value = serde_json::from_slice(&output2).unwrap();
        // Cursor approve → {"action": "allow"}
        assert_eq!(json2["action"], "allow");
    }
}
