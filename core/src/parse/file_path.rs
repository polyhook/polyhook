//! Normalized file path of write and edit tools.

/// Input keys that hold the path of the file a tool writes. Each agent uses
/// a different one, so the first key that holds a string wins.
const FILE_PATH_KEYS: &[&str] = &["file_path", "path", "notebook_path", "target_file"];

pub(super) fn extract_file_path(
    tool: Option<&str>,
    input: Option<&serde_json::Map<String, serde_json::Value>>,
) -> Option<String> {
    if !matches!(tool, Some("write_file" | "edit_file")) {
        return None;
    }
    let input = input?;
    FILE_PATH_KEYS
        .iter()
        .find_map(|key| input.get(*key).and_then(|v| v.as_str()))
        .map(str::to_owned)
}
