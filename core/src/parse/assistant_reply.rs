//! Final reply text and transcript path for `turn:stop` events.

use super::json_string_field::str_field;
use crate::types::CallerKind;

pub(super) fn extract_transcript_path(val: &serde_json::Value) -> Option<String> {
    str_field(val, "transcript_path").map(str::to_owned)
}

pub(super) fn extract_reply(
    val: &serde_json::Value,
    caller: CallerKind,
    transcript_path: Option<&str>,
) -> Option<String> {
    let reply = match caller {
        CallerKind::GeminiCli => str_field(val, "prompt_response"),
        CallerKind::Hermes => val
            .get("extra")
            .and_then(|e| e.get("assistant_response"))
            .and_then(|r| r.as_str()),
        _ => str_field(val, "last_assistant_message"),
    };
    match (reply, caller) {
        (Some(r), _) => Some(r.to_owned()),
        // Older Claude Code builds send no `last_assistant_message`; read it
        // from the transcript. On wasm32-unknown-unknown std::fs always fails,
        // so WASM SDKs get None here.
        (None, CallerKind::ClaudeCode | CallerKind::Codex | CallerKind::Pi) => {
            last_assistant_text(&std::fs::read_to_string(transcript_path?).ok()?)
        }
        _ => None,
    }
}

/// Text blocks of the last transcript entry with `type: "assistant"` that
/// holds any text, joined by newlines.
fn last_assistant_text(jsonl: &str) -> Option<String> {
    jsonl.lines().rev().find_map(|line| {
        let entry: serde_json::Value = serde_json::from_str(line).ok()?;
        if str_field(&entry, "type") != Some("assistant") {
            return None;
        }
        let texts: Vec<&str> = entry
            .pointer("/message/content")?
            .as_array()?
            .iter()
            .filter(|b| str_field(b, "type") == Some("text"))
            .filter_map(|b| str_field(b, "text"))
            .collect();
        (!texts.is_empty()).then(|| texts.join("\n"))
    })
}

#[cfg(test)]
#[path = "assistant_reply_tests.rs"]
mod tests;
