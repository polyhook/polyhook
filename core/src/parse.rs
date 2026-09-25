use crate::detect::detect_caller;
use crate::events::normalize_event;
use crate::tools::normalize_tool;
use crate::types::{CallerKind, HookEvent, HookEventEvent};

/// Parse raw stdin bytes into a normalized [`HookEvent`].
pub fn parse_event(raw: &[u8]) -> Result<HookEvent, String> {
    let val: serde_json::Value =
        serde_json::from_slice(raw).map_err(|e| format!("JSON parse error: {e}"))?;

    let caller = detect_caller(&val);

    // --- tool name ---
    let raw_tool = extract_tool_field(&val, caller);
    let tool = raw_tool.map(|t| normalize_tool(&t, &caller));

    // --- input / output ---
    let input = extract_input(&val, caller);
    let output = extract_output(&val, caller);

    // --- event name ---
    // When the caller's event field is absent (e.g. a raw tool payload piped
    // without its `hook_event_name` envelope) or carries an unrecognized
    // value, fall back to inferring the event from the payload shape rather
    // than blindly labelling it a notification. A detected tool with no output
    // is a pending call (`tool:before`); with output it has already run
    // (`tool:after`); anything else stays a notification.
    let raw_event = extract_event_field(&val, caller);
    let event = if raw_event.is_empty() {
        infer_event(&tool, &output)
    } else {
        normalize_event(&raw_event, &caller)
            .parse::<HookEventEvent>()
            .unwrap_or_else(|_| infer_event(&tool, &output))
    };

    let prompt = if event == HookEventEvent::PromptSubmit {
        extract_prompt(&val, caller)
    } else {
        None
    };

    // --- session / agent ids ---
    let session_id = extract_session_id(&val);
    let agent_id = extract_agent_id(&val);

    Ok(HookEvent {
        event,
        tool,
        input,
        output,
        session_id,
        agent_id,
        caller,
        prompt,
    })
}

/// Infer the normalized event kind from the payload shape when no usable
/// vendor event name is available. A recognized tool with no output is a
/// pending tool call; with output it has already run; otherwise there is
/// nothing actionable, so treat it as a notification.
fn infer_event(
    tool: &Option<String>,
    output: &Option<serde_json::Map<String, serde_json::Value>>,
) -> HookEventEvent {
    match (tool, output) {
        (Some(_), Some(_)) => HookEventEvent::ToolAfter,
        (Some(_), None) => HookEventEvent::ToolBefore,
        _ => HookEventEvent::Notification,
    }
}

// ---------------------------------------------------------------------------
// Field extraction helpers
// ---------------------------------------------------------------------------

fn str_field<'a>(val: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    val.get(key).and_then(|v| v.as_str())
}

fn extract_event_field(val: &serde_json::Value, caller: CallerKind) -> String {
    let candidates: &[&str] = match caller {
        CallerKind::ClaudeCode | CallerKind::Pi | CallerKind::Codex => &[
            "hook_event_name",
            "event",
            "hookEvent",
            "hook_event",
            "type",
        ],
        CallerKind::Cursor => &["type", "event"],
        CallerKind::Windsurf => &["event", "type"],
        CallerKind::Cline => &["hookName", "type", "event"],
        CallerKind::Amp => &["kind", "event", "type"],
        CallerKind::GeminiCli => &["hook_event_name"],
        CallerKind::Hermes => &["hook_event_name"],
        CallerKind::Unknown => &["event", "type", "kind", "hookEvent"],
    };

    for key in candidates {
        if let Some(s) = str_field(val, key) {
            return s.to_owned();
        }
    }
    String::new()
}

fn extract_tool_field(val: &serde_json::Value, caller: CallerKind) -> Option<String> {
    match caller {
        CallerKind::ClaudeCode | CallerKind::Pi | CallerKind::Codex => {
            str_field(val, "tool_name").map(str::to_owned)
        }
        CallerKind::Cursor => val
            .get("toolCall")
            .and_then(|tc| tc.get("name"))
            .and_then(|n| n.as_str())
            .map(str::to_owned),
        CallerKind::Windsurf => str_field(val, "tool").map(str::to_owned),
        CallerKind::Cline => str_field(val, "toolName").map(str::to_owned),
        CallerKind::Amp => str_field(val, "name").map(str::to_owned),
        CallerKind::GeminiCli => str_field(val, "tool_name").map(str::to_owned),
        CallerKind::Hermes => str_field(val, "tool_name").map(str::to_owned),
        CallerKind::Unknown => {
            for key in &["tool_name", "toolName", "tool", "name"] {
                if let Some(s) = str_field(val, key) {
                    return Some(s.to_owned());
                }
            }
            None
        }
    }
}

fn into_map(v: serde_json::Value) -> Option<serde_json::Map<String, serde_json::Value>> {
    match v {
        serde_json::Value::Object(m) => Some(m),
        _ => None,
    }
}

fn extract_input(
    val: &serde_json::Value,
    caller: CallerKind,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    let raw = match caller {
        CallerKind::ClaudeCode | CallerKind::Pi | CallerKind::Codex => {
            val.get("tool_input").cloned()
        }
        CallerKind::Cursor => val.get("toolCall").and_then(|tc| tc.get("args")).cloned(),
        CallerKind::Windsurf => val.get("parameters").cloned(),
        CallerKind::Cline => val
            .get("args")
            .cloned()
            .or_else(|| val.get("input").cloned()),
        CallerKind::Amp => val
            .get("args")
            .cloned()
            .or_else(|| val.get("input").cloned()),
        CallerKind::GeminiCli => val.get("tool_input").cloned(),
        CallerKind::Hermes => val.get("tool_input").cloned(),
        CallerKind::Unknown => {
            for key in &["tool_input", "args", "parameters", "input"] {
                if let Some(v) = val.get(key) {
                    return into_map(v.clone());
                }
            }
            None
        }
    };
    raw.and_then(into_map)
}

fn extract_output(
    val: &serde_json::Value,
    caller: CallerKind,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    let raw = match caller {
        CallerKind::ClaudeCode | CallerKind::Pi | CallerKind::Codex => {
            val.get("tool_output").cloned()
        }
        CallerKind::Cursor => val.get("toolCall").and_then(|tc| tc.get("result")).cloned(),
        CallerKind::Windsurf => val.get("result").cloned(),
        CallerKind::Cline => val
            .get("result")
            .cloned()
            .or_else(|| val.get("output").cloned()),
        CallerKind::Amp => val
            .get("result")
            .cloned()
            .or_else(|| val.get("output").cloned()),
        CallerKind::GeminiCli => val.get("tool_response").cloned(),
        CallerKind::Hermes => val
            .get("tool_output")
            .cloned()
            .or_else(|| val.get("tool_response").cloned()),
        CallerKind::Unknown => {
            for key in &["tool_output", "result", "output"] {
                if let Some(v) = val.get(key) {
                    return into_map(v.clone());
                }
            }
            None
        }
    };
    raw.and_then(into_map)
}

fn extract_session_id(val: &serde_json::Value) -> String {
    for key in &["session_id", "sessionId", "session"] {
        if let Some(s) = str_field(val, key) {
            return s.to_owned();
        }
    }
    String::new()
}

fn extract_prompt(val: &serde_json::Value, caller: CallerKind) -> Option<String> {
    let prompt = match caller {
        CallerKind::Hermes => val.get("extra").and_then(|e| e.get("user_message")),
        CallerKind::Cline => val.get("userPromptSubmit").and_then(|p| p.get("prompt")),
        _ => val.get("prompt"),
    };
    prompt.and_then(|p| p.as_str()).map(str::to_owned)
}

fn extract_agent_id(val: &serde_json::Value) -> Option<String> {
    for key in &["agent_id", "agentId", "agent"] {
        if let Some(s) = str_field(val, key) {
            return Some(s.to_owned());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "parse_tests.rs"]
mod tests;
