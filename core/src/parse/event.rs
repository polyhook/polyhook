//! Vendor event name lookup and shape-based event inference.

use super::field::str_field;
use crate::types::{CallerKind, HookEventEvent};

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

/// Infer the normalized event kind from the payload shape when no usable
/// vendor event name is available. A recognized tool with no output is a
/// pending tool call; with output it has already run; otherwise there is
/// nothing actionable, so treat it as a notification.
pub(super) fn infer_event(
    tool: &Option<String>,
    output: &Option<serde_json::Map<String, serde_json::Value>>,
) -> HookEventEvent {
    match (tool, output) {
        (Some(_), Some(_)) => HookEventEvent::ToolAfter,
        (Some(_), None) => HookEventEvent::ToolBefore,
        _ => HookEventEvent::Notification,
    }
}
