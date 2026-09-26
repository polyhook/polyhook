//! Vendor tool name lookup.

use super::field::str_field;
use crate::types::CallerKind;

pub(super) fn extract_tool_field(val: &serde_json::Value, caller: CallerKind) -> Option<String> {
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
