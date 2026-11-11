//! Windsurf Cascade's real hook payloads: the event lives in
//! `agent_action_name` and everything else in `tool_info`
//! (https://docs.windsurf.com/windsurf/cascade/hooks).

use super::json_string_field::str_field;
use serde_json::Value;

fn tool_info(val: &Value) -> Option<&Value> {
    val.get("tool_info")
}

/// Tool name implied by the action, in Windsurf's own vocabulary so
/// `normalize_tool` maps it. MCP calls use the `mcp__<server>__<tool>` form.
pub(super) fn action_tool(val: &Value) -> Option<String> {
    let action = str_field(val, "agent_action_name")?;
    let kind = action
        .strip_prefix("pre_")
        .or_else(|| action.strip_prefix("post_"))?;
    match kind {
        "run_command" => Some("run_command".to_owned()),
        "read_code" => Some("read_file".to_owned()),
        "write_code" => Some("edit_file".to_owned()),
        "mcp_tool_use" => {
            let info = tool_info(val)?;
            let server = str_field(info, "mcp_server_name")?;
            let tool = str_field(info, "mcp_tool_name")?;
            Some(format!("mcp__{server}__{tool}"))
        }
        _ => None,
    }
}

/// Tool input from `tool_info`: MCP arguments for MCP calls; otherwise the
/// whole object, with `command_line` also exposed as the canonical `command`.
pub(super) fn action_input(val: &Value) -> Option<Value> {
    let info = tool_info(val)?;
    if let Some(args) = info.get("mcp_tool_arguments") {
        return Some(args.clone());
    }
    let mut input = info.clone();
    if let (Some(map), Some(cmd)) = (input.as_object_mut(), info.get("command_line")) {
        map.insert("command".to_owned(), cmd.clone());
    }
    Some(input)
}

#[cfg(test)]
#[path = "windsurf_tests.rs"]
mod tests;
