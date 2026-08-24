use crate::types::{CallerKind, HookEventEvent, HookResponse};
use serde_json::{json, Value};

/// Serialize a [`HookResponse`] into the JSON format expected by the detected caller.
///
/// Does not use event-type context. For PreToolUse blocking on Claude Code, prefer
/// [`serialize_response_with_event`] so the correct `hookSpecificOutput` format is used.
pub fn serialize_response(resp: &HookResponse, caller: &CallerKind) -> Value {
    serialize_response_with_event(resp, *caller, None)
}

/// Like [`serialize_response`] but uses the event type to pick the correct block format.
/// For Claude Code + `tool:before`, emits `hookSpecificOutput.permissionDecision: "deny"`
/// instead of `decision: "block"` (which would terminate the whole session).
pub(crate) fn serialize_response_with_event(
    resp: &HookResponse,
    caller: CallerKind,
    event: Option<HookEventEvent>,
) -> Value {
    match caller {
        CallerKind::ClaudeCode | CallerKind::Pi | CallerKind::Unknown => {
            serialize_claude_code(resp, event)
        }
        CallerKind::Cursor => serialize_cursor(resp),
        CallerKind::Windsurf => serialize_windsurf(resp),
        CallerKind::Cline => serialize_cline(resp),
        CallerKind::Amp => serialize_amp(resp),
        CallerKind::GeminiCli => serialize_gemini_cli(resp),
        CallerKind::Hermes => serialize_hermes(resp),
    }
}

// ---------------------------------------------------------------------------
// Per-caller serializers
// ---------------------------------------------------------------------------

fn serialize_claude_code(resp: &HookResponse, event: Option<HookEventEvent>) -> Value {
    let is_pre_tool_use = matches!(event, Some(HookEventEvent::ToolBefore));
    match resp {
        HookResponse::ApproveResponse(_) => {
            if is_pre_tool_use {
                // PreToolUse only reads hookSpecificOutput.permissionDecision; an empty
                // `{}` is a passive no-op (falls through to the normal permission flow)
                // rather than an explicit allow that bypasses the confirmation prompt.
                json!({
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": "allow"
                    }
                })
            } else {
                json!({})
            }
        }
        HookResponse::BlockResponse(b) => {
            if is_pre_tool_use {
                json!({
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": "deny",
                        "permissionDecisionReason": b.message
                    }
                })
            } else {
                json!({ "decision": "block", "reason": b.message })
            }
        }
        HookResponse::ModifyResponse(m) => {
            if is_pre_tool_use {
                // Same reasoning as BlockResponse above: PreToolUse does not read the
                // top-level `decision`/`tool_input` fields, so the replacement input
                // must go under hookSpecificOutput.updatedInput to actually apply.
                json!({
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": "allow",
                        "updatedInput": m.input
                    }
                })
            } else {
                json!({ "decision": "approve", "tool_input": m.input })
            }
        }
    }
}

/// Cursor's real hook response schema (https://cursor.com/docs/hooks) is
/// `permission: "allow" | "deny" | "ask"` plus optional `user_message` /
/// `agent_message`, shared by every hook type that reads a response
/// (preToolUse, beforeShellExecution, beforeMCPExecution, beforeReadFile, ...).
/// Hooks that don't support a response (afterFileEdit, sessionEnd, ...) simply
/// ignore these fields, so one shape covers every event.
fn serialize_cursor(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({ "permission": "allow" }),
        HookResponse::BlockResponse(b) => {
            json!({
                "permission": "deny",
                "user_message": b.message,
                "agent_message": b.message
            })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "permission": "allow", "updated_input": m.input })
        }
    }
}

fn serialize_windsurf(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({ "allow": true }),
        HookResponse::BlockResponse(b) => {
            json!({ "allow": false, "reason": b.message })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "allow": true, "modified_parameters": m.input })
        }
    }
}

fn serialize_cline(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({ "approved": true }),
        HookResponse::BlockResponse(b) => {
            json!({ "approved": false, "reason": b.message })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "approved": true, "modifiedInput": m.input })
        }
    }
}

fn serialize_amp(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({ "result": "allow" }),
        HookResponse::BlockResponse(b) => {
            json!({ "result": "deny", "reason": b.message })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "result": "allow", "modified": m.input })
        }
    }
}

fn serialize_gemini_cli(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({ "decision": "allow" }),
        HookResponse::BlockResponse(b) => {
            json!({ "decision": "deny", "reason": b.message })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "decision": "allow", "tool_input": m.input })
        }
    }
}

fn serialize_hermes(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({}),
        HookResponse::BlockResponse(b) => {
            json!({ "action": "block", "message": b.message })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "action": "modify", "tool_input": m.input })
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "response_tests.rs"]
mod tests;
