//! Session and agent identifiers.

use super::json_string_field::str_field;

pub(super) fn extract_session_id(val: &serde_json::Value) -> String {
    for key in &["session_id", "sessionId", "session"] {
        if let Some(s) = str_field(val, key) {
            return s.to_owned();
        }
    }
    String::new()
}

pub(super) fn extract_agent_id(val: &serde_json::Value) -> Option<String> {
    for key in &["agent_id", "agentId", "agent"] {
        if let Some(s) = str_field(val, key) {
            return Some(s.to_owned());
        }
    }
    None
}
