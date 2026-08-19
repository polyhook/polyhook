use super::*;

// Helper: read a length-prefixed buffer returned by parse/serialize.
// Returns the payload bytes and the total buffer length (4 + payload).
unsafe fn read_length_prefixed(ptr: *mut u8) -> (Vec<u8>, usize) {
    // Read 4-byte LE length header.
    let len_bytes: [u8; 4] = std::slice::from_raw_parts(ptr, 4)
        .try_into()
        .expect("slice to array");
    let payload_len = i32::from_le_bytes(len_bytes) as usize;
    let total = 4 + payload_len;

    // Copy out the payload.
    let payload = std::slice::from_raw_parts(ptr.add(4), payload_len).to_vec();
    (payload, total)
}

// ---------------------------------------------------------------------------
// alloc / dealloc
// ---------------------------------------------------------------------------

#[test]
fn alloc_zero_does_not_crash() {
    unsafe {
        let ptr = alloc(0);
        // Deallocating a zero-length allocation; ptr may be dangling/null but
        // Vec::from_raw_parts(ptr, 0, 0) is defined to drop nothing.
        dealloc(ptr, 0);
    }
}

#[test]
fn alloc_returns_non_null_for_nonzero_len() {
    unsafe {
        let ptr = alloc(64);
        assert!(!ptr.is_null());
        dealloc(ptr, 64);
    }
}

#[test]
fn dealloc_of_alloc_does_not_crash() {
    unsafe {
        let ptr = alloc(128);
        assert!(!ptr.is_null());
        // Write something to confirm the memory is usable.
        std::ptr::write_bytes(ptr, 0xAB, 128);
        dealloc(ptr, 128);
    }
}

// ---------------------------------------------------------------------------
// parse
// ---------------------------------------------------------------------------

#[test]
fn parse_valid_claude_code_json_returns_tool_before() {
    let json =
        br#"{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"},"session_id":"s1"}"#;
    unsafe {
        let input_ptr = alloc(json.len());
        std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());

        let out_ptr = parse(input_ptr as *const u8, json.len());
        assert!(!out_ptr.is_null());

        let (payload, total) = read_length_prefixed(out_ptr);
        let s = String::from_utf8(payload).expect("valid utf8");
        assert!(s.contains("tool:before"), "expected 'tool:before' in: {s}");

        dealloc(input_ptr, json.len());
        dealloc(out_ptr, total);
    }
}

#[test]
fn set_env_polyhook_caller_is_honoured_by_parse() {
    let env = br#"{"POLYHOOK_CALLER":"claude-code"}"#;
    let json = br#"{"session_id":"s","hook_event_name":"Stop"}"#;
    unsafe {
        let env_ptr = alloc(env.len());
        std::ptr::copy_nonoverlapping(env.as_ptr(), env_ptr, env.len());
        set_env(env_ptr as *const u8, env.len());
        dealloc(env_ptr, env.len());

        let input_ptr = alloc(json.len());
        std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());
        let out_ptr = parse(input_ptr as *const u8, json.len());
        let (payload, total) = read_length_prefixed(out_ptr);
        let s = String::from_utf8(payload).expect("valid utf8");
        dealloc(input_ptr, json.len());
        dealloc(out_ptr, total);

        // Reset so other tests are not affected.
        set_env(b"{}".as_ptr(), 2);

        assert!(s.contains("\"caller\":\"claude-code\""), "in: {s}");
        assert!(s.contains("session:stop"), "in: {s}");
    }
}

#[test]
fn set_env_malformed_input_clears_env() {
    let bad = b"nope";
    let json = br#"{"session_id":"s","hook_event_name":"Stop"}"#;
    unsafe {
        set_env(bad.as_ptr(), bad.len());
        let input_ptr = alloc(json.len());
        std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());
        let out_ptr = parse(input_ptr as *const u8, json.len());
        let (payload, total) = read_length_prefixed(out_ptr);
        let s = String::from_utf8(payload).expect("valid utf8");
        dealloc(input_ptr, json.len());
        dealloc(out_ptr, total);
        assert!(s.contains("\"caller\":\"unknown\""), "in: {s}");
    }
}

#[test]
fn parse_invalid_json_returns_error_payload() {
    let bad = b"this is not json";
    unsafe {
        let input_ptr = alloc(bad.len());
        std::ptr::copy_nonoverlapping(bad.as_ptr(), input_ptr, bad.len());

        let out_ptr = parse(input_ptr as *const u8, bad.len());
        assert!(!out_ptr.is_null());

        let (payload, total) = read_length_prefixed(out_ptr);
        let s = String::from_utf8(payload).expect("valid utf8");
        assert!(s.contains("error"), "expected 'error' in: {s}");

        dealloc(input_ptr, bad.len());
        dealloc(out_ptr, total);
    }
}

// ---------------------------------------------------------------------------
// serialize
// ---------------------------------------------------------------------------

#[test]
fn serialize_approve_returns_length_prefixed_json() {
    let json = br#"{"action":"approve"}"#;
    unsafe {
        let input_ptr = alloc(json.len());
        std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());

        let out_ptr = serialize(input_ptr as *const u8, json.len());
        assert!(!out_ptr.is_null());

        let (payload, total) = read_length_prefixed(out_ptr);
        let s = String::from_utf8(payload).expect("valid utf8");
        // Should be valid JSON and not contain "error".
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("should be valid JSON");
        assert!(!s.contains("error"), "unexpected error in: {s}");
        // Result is a JSON object.
        assert!(parsed.is_object());

        dealloc(input_ptr, json.len());
        dealloc(out_ptr, total);
    }
}

#[test]
fn serialize_block_for_pre_tool_use_uses_permission_decision_deny() {
    // Parse a Claude Code PreToolUse event so LAST_CALLER/LAST_EVENT are set
    // to (ClaudeCode, ToolBefore).
    let event_json =
        br#"{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"},"session_id":"s2"}"#;
    unsafe {
        let ep = alloc(event_json.len());
        std::ptr::copy_nonoverlapping(event_json.as_ptr(), ep, event_json.len());
        let ep_out = parse(ep as *const u8, event_json.len());
        let (_, ep_total) = read_length_prefixed(ep_out);
        dealloc(ep, event_json.len());
        dealloc(ep_out, ep_total);
    }

    let json = br#"{"action":"block","message":"dangerous command"}"#;
    unsafe {
        let input_ptr = alloc(json.len());
        std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());

        let out_ptr = serialize(input_ptr as *const u8, json.len());
        assert!(!out_ptr.is_null());

        let (payload, total) = read_length_prefixed(out_ptr);
        let s = String::from_utf8(payload).expect("valid utf8");
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("should be valid JSON");
        // Blocking a PreToolUse call must use hookSpecificOutput/permissionDecision
        // so Claude Code denies only this tool call, not `decision: "block"` which
        // terminates the whole session (see issue #11 — this is the WASM ABI
        // path that fix originally missed).
        assert_eq!(
            parsed["hookSpecificOutput"]["hookEventName"],
            serde_json::json!("PreToolUse")
        );
        assert_eq!(
            parsed["hookSpecificOutput"]["permissionDecision"],
            serde_json::json!("deny")
        );
        assert_eq!(
            parsed["hookSpecificOutput"]["permissionDecisionReason"],
            serde_json::json!("dangerous command")
        );
        assert!(parsed.get("decision").is_none());

        dealloc(input_ptr, json.len());
        dealloc(out_ptr, total);
    }
}

#[test]
fn serialize_block_for_post_tool_use_uses_decision_block() {
    // Parse a Claude Code PostToolUse event so LAST_CALLER/LAST_EVENT are set
    // to (ClaudeCode, ToolAfter) — this variant still uses the generic
    // `decision: "block"` format, which is correct outside PreToolUse.
    let event_json = br#"{"type":"PostToolUse","tool_name":"Bash","tool_input":{"command":"ls"},"tool_output":{"content":"ok"},"session_id":"s2"}"#;
    unsafe {
        let ep = alloc(event_json.len());
        std::ptr::copy_nonoverlapping(event_json.as_ptr(), ep, event_json.len());
        let ep_out = parse(ep as *const u8, event_json.len());
        let (_, ep_total) = read_length_prefixed(ep_out);
        dealloc(ep, event_json.len());
        dealloc(ep_out, ep_total);
    }

    let json = br#"{"action":"block","message":"dangerous command"}"#;
    unsafe {
        let input_ptr = alloc(json.len());
        std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());

        let out_ptr = serialize(input_ptr as *const u8, json.len());
        assert!(!out_ptr.is_null());

        let (payload, total) = read_length_prefixed(out_ptr);
        let s = String::from_utf8(payload).expect("valid utf8");
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("should be valid JSON");
        assert_eq!(parsed["decision"], serde_json::json!("block"));
        assert_eq!(parsed["reason"], serde_json::json!("dangerous command"));
        assert!(parsed.get("hookSpecificOutput").is_none());

        dealloc(input_ptr, json.len());
        dealloc(out_ptr, total);
    }
}

#[test]
fn serialize_modify_returns_length_prefixed_json() {
    // Ensure LAST_CALLER is set via parse.
    let event_json =
        br#"{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"},"session_id":"s3"}"#;
    unsafe {
        let ep = alloc(event_json.len());
        std::ptr::copy_nonoverlapping(event_json.as_ptr(), ep, event_json.len());
        let ep_out = parse(ep as *const u8, event_json.len());
        let (_, ep_total) = read_length_prefixed(ep_out);
        dealloc(ep, event_json.len());
        dealloc(ep_out, ep_total);
    }

    let json = br#"{"action":"modify","input":{"command":"echo safe"}}"#;
    unsafe {
        let input_ptr = alloc(json.len());
        std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());

        let out_ptr = serialize(input_ptr as *const u8, json.len());
        assert!(!out_ptr.is_null());

        let (payload, total) = read_length_prefixed(out_ptr);
        let s = String::from_utf8(payload).expect("valid utf8");
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("should be valid JSON");
        assert!(parsed.is_object());
        assert!(!s.contains("error"), "unexpected error in: {s}");

        dealloc(input_ptr, json.len());
        dealloc(out_ptr, total);
    }
}

#[test]
fn serialize_invalid_json_returns_error_payload() {
    let bad = b"not json at all!";
    unsafe {
        let input_ptr = alloc(bad.len());
        std::ptr::copy_nonoverlapping(bad.as_ptr(), input_ptr, bad.len());

        let out_ptr = serialize(input_ptr as *const u8, bad.len());
        assert!(!out_ptr.is_null());

        let (payload, total) = read_length_prefixed(out_ptr);
        let s = String::from_utf8(payload).expect("valid utf8");
        assert!(s.contains("error"), "expected 'error' in: {s}");

        dealloc(input_ptr, bad.len());
        dealloc(out_ptr, total);
    }
}

// ---------------------------------------------------------------------------
// length_prefix_alloc (private helper exercised indirectly above, but also
// tested directly via the public API round-trip)
// ---------------------------------------------------------------------------

#[test]
fn length_prefix_alloc_encodes_correct_length() {
    let payload = b"hello world";
    let payload_vec = payload.to_vec();
    let payload_len = payload_vec.len();

    unsafe {
        let ptr = length_prefix_alloc(payload_vec);
        assert!(!ptr.is_null());

        // Read back the 4-byte LE header.
        let len_bytes: [u8; 4] = std::slice::from_raw_parts(ptr, 4).try_into().unwrap();
        let decoded_len = i32::from_le_bytes(len_bytes) as usize;
        assert_eq!(decoded_len, payload_len);

        // Verify payload bytes.
        let actual_payload = std::slice::from_raw_parts(ptr.add(4), payload_len);
        assert_eq!(actual_payload, payload);

        dealloc(ptr, 4 + payload_len);
    }
}
