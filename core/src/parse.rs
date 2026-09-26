mod event;
mod field;
mod ids;
mod payload;
mod prompt;
mod tool;

use crate::detect::detect_caller;
use crate::events::normalize_event;
use crate::tools::normalize_tool;
use crate::types::{HookEvent, HookEventEvent};
use event::{extract_event_field, infer_event};
use ids::{extract_agent_id, extract_session_id};
use payload::{extract_input, extract_output};
use prompt::extract_prompt;
use tool::extract_tool_field;

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "parse_tests.rs"]
mod tests;
