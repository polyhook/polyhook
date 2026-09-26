//! Event kind inference from the payload shape, for payloads without a usable event name.

use crate::types::HookEventEvent;

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
