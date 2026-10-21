//! Tool input and output extraction.

use super::windsurf;
use crate::types::CallerKind;

pub(super) fn into_map(v: serde_json::Value) -> Option<serde_json::Map<String, serde_json::Value>> {
    match v {
        serde_json::Value::Object(m) => Some(m),
        _ => None,
    }
}

pub(super) fn extract_input(
    val: &serde_json::Value,
    caller: CallerKind,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    let raw = match caller {
        CallerKind::ClaudeCode | CallerKind::Pi | CallerKind::Codex => {
            val.get("tool_input").cloned()
        }
        CallerKind::Cursor => val.get("toolCall").and_then(|tc| tc.get("args")).cloned(),
        CallerKind::Windsurf => val
            .get("parameters")
            .cloned()
            .or_else(|| windsurf::action_input(val)),
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

pub(super) fn extract_output(
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
