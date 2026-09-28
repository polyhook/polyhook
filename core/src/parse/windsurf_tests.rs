use crate::parse::parse_event;
use crate::CallerKind;
use serde_json::json;

fn parse(val: serde_json::Value) -> crate::HookEvent {
    parse_event(val.to_string().as_bytes()).expect("parse failed")
}

fn envelope(action: &str, tool_info: serde_json::Value) -> serde_json::Value {
    json!({
        "agent_action_name": action,
        "trajectory_id": "trajectory_1",
        "execution_id": "exec_1",
        "timestamp": "2026-09-28T00:00:00Z",
        "model_name": "swe-1",
        "tool_info": tool_info
    })
}

#[test]
fn pre_run_command_is_bash_tool_before() {
    let evt = parse(envelope(
        "pre_run_command",
        json!({ "command_line": "git status", "cwd": "/tmp" }),
    ));
    assert_eq!(evt.caller, CallerKind::Windsurf);
    assert_eq!(evt.event.to_string(), "tool:before");
    assert_eq!(evt.tool.as_deref(), Some("bash"));
    assert_eq!(evt.session_id, "trajectory_1");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["command"], json!("git status"));
    assert_eq!(input["cwd"], json!("/tmp"));
}

#[test]
fn post_run_command_is_tool_after() {
    let evt = parse(envelope(
        "post_run_command",
        json!({ "command_line": "ls", "cwd": "/tmp" }),
    ));
    assert_eq!(evt.event.to_string(), "tool:after");
    assert_eq!(evt.tool.as_deref(), Some("bash"));
}

#[test]
fn pre_mcp_tool_use_uses_mcp_name_and_arguments() {
    let evt = parse(envelope(
        "pre_mcp_tool_use",
        json!({
            "mcp_server_name": "github",
            "mcp_tool_name": "push_files",
            "mcp_tool_arguments": { "owner": "o", "repo": "r" }
        }),
    ));
    assert_eq!(evt.event.to_string(), "tool:before");
    assert_eq!(evt.tool.as_deref(), Some("mcp__github__push_files"));
    assert_eq!(evt.input.expect("input")["repo"], json!("r"));
}

#[test]
fn read_and_write_code_map_to_file_tools() {
    let read = parse(envelope("pre_read_code", json!({ "file_path": "/a.rs" })));
    assert_eq!(read.tool.as_deref(), Some("read_file"));
    assert_eq!(read.input.expect("input")["file_path"], json!("/a.rs"));
    let write = parse(envelope("post_write_code", json!({ "file_path": "/a.rs" })));
    assert_eq!(write.event.to_string(), "tool:after");
    assert_eq!(write.tool.as_deref(), Some("edit_file"));
}

#[test]
fn non_tool_action_has_no_tool() {
    let evt = parse(envelope(
        "post_cascade_response",
        json!({ "response": "hi" }),
    ));
    assert_eq!(evt.caller, CallerKind::Windsurf);
    assert!(evt.tool.is_none());
    assert_eq!(evt.event.to_string(), "notification");
}
