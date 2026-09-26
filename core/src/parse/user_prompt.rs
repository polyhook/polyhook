//! User prompt extraction for `prompt:submit` events.

use crate::types::CallerKind;

pub(super) fn extract_prompt(val: &serde_json::Value, caller: CallerKind) -> Option<String> {
    let prompt = match caller {
        CallerKind::Hermes => val.get("extra").and_then(|e| e.get("user_message")),
        CallerKind::Cline => val.get("userPromptSubmit").and_then(|p| p.get("prompt")),
        _ => val.get("prompt"),
    };
    prompt.and_then(|p| p.as_str()).map(str::to_owned)
}
