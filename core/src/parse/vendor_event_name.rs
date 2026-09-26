//! Vendor event name lookup, tried in each caller's field order.

use super::json_string_field::str_field;
use crate::types::CallerKind;

pub(super) fn extract_event_field(val: &serde_json::Value, caller: CallerKind) -> String {
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
