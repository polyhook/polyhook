//! Session context fields: working directory and model slug.

use super::json_string_field::str_field;

pub(super) fn extract_cwd(val: &serde_json::Value) -> Option<String> {
    str_field(val, "cwd").map(str::to_owned)
}

pub(super) fn extract_model(val: &serde_json::Value) -> Option<String> {
    str_field(val, "model").map(str::to_owned)
}
