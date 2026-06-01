//! Coverage tests for gaps identified in events.rs, tools.rs, and type_impls.rs.

use polyhook_core::events::normalize_event;
use polyhook_core::tools::normalize_tool;
use polyhook_core::types::{CallerKind, HookResponse};

// ---------------------------------------------------------------------------
// normalize_event — CallerKind::Unknown branch
// ---------------------------------------------------------------------------

#[test]
fn normalize_event_unknown_caller_returns_original() {
    // With CallerKind::Unknown the function returns None from the match arm,
    // so unwrap_or_else gives back the original string unchanged.
    assert_eq!(
        normalize_event("SomeVendorEvent", &CallerKind::Unknown),
        "SomeVendorEvent"
    );
    assert_eq!(
        normalize_event("PreToolUse", &CallerKind::Unknown),
        "PreToolUse"
    );
    assert_eq!(normalize_event("", &CallerKind::Unknown), "");
}

// ---------------------------------------------------------------------------
// normalize_tool — CallerKind::Unknown branch
// ---------------------------------------------------------------------------

#[test]
fn normalize_tool_unknown_caller_returns_original() {
    // With CallerKind::Unknown normalize_tool returns None, so the original
    // vendor string is returned unchanged.
    assert_eq!(
        normalize_tool("SomeTool", &CallerKind::Unknown),
        "SomeTool"
    );
    assert_eq!(normalize_tool("bash", &CallerKind::Unknown), "bash");
    assert_eq!(normalize_tool("Bash", &CallerKind::Unknown), "Bash");
}

// ---------------------------------------------------------------------------
// normalize_tool — ClaudeCode mappings missing from integration_test.rs
// ---------------------------------------------------------------------------

#[test]
fn normalize_tool_claude_code_ls_maps_to_list_dir() {
    assert_eq!(normalize_tool("ls", &CallerKind::ClaudeCode), "list_dir");
    // Case-insensitive: "LS" should also map.
    assert_eq!(normalize_tool("LS", &CallerKind::ClaudeCode), "list_dir");
}

#[test]
fn normalize_tool_claude_code_grep_maps_to_grep() {
    assert_eq!(normalize_tool("grep", &CallerKind::ClaudeCode), "grep");
    assert_eq!(normalize_tool("Grep", &CallerKind::ClaudeCode), "grep");
}

#[test]
fn normalize_tool_claude_code_glob_maps_to_glob() {
    assert_eq!(normalize_tool("glob", &CallerKind::ClaudeCode), "glob");
    assert_eq!(normalize_tool("Glob", &CallerKind::ClaudeCode), "glob");
}

#[test]
fn normalize_tool_claude_code_websearch_maps_to_web_search() {
    assert_eq!(
        normalize_tool("websearch", &CallerKind::ClaudeCode),
        "web_search"
    );
    assert_eq!(
        normalize_tool("WebSearch", &CallerKind::ClaudeCode),
        "web_search"
    );
}

#[test]
fn normalize_tool_claude_code_webfetch_maps_to_web_fetch() {
    assert_eq!(
        normalize_tool("webfetch", &CallerKind::ClaudeCode),
        "web_fetch"
    );
    assert_eq!(
        normalize_tool("WebFetch", &CallerKind::ClaudeCode),
        "web_fetch"
    );
}

#[test]
fn normalize_tool_claude_code_mcp_ide_getdiagnostics_maps_to_diagnostics() {
    assert_eq!(
        normalize_tool("mcp__ide__getdiagnostics", &CallerKind::ClaudeCode),
        "diagnostics"
    );
    assert_eq!(
        normalize_tool("MCP__IDE__GetDiagnostics", &CallerKind::ClaudeCode),
        "diagnostics"
    );
}

// ---------------------------------------------------------------------------
// normalize_tool — Cursor mappings missing from integration_test.rs
// ---------------------------------------------------------------------------

#[test]
fn normalize_tool_cursor_apply_edit_maps_to_edit_file() {
    assert_eq!(
        normalize_tool("apply_edit", &CallerKind::Cursor),
        "edit_file"
    );
}

#[test]
fn normalize_tool_cursor_list_dir_maps_to_list_dir() {
    assert_eq!(
        normalize_tool("list_dir", &CallerKind::Cursor),
        "list_dir"
    );
}

#[test]
fn normalize_tool_cursor_file_search_maps_to_glob() {
    assert_eq!(
        normalize_tool("file_search", &CallerKind::Cursor),
        "glob"
    );
}

#[test]
fn normalize_tool_cursor_fetch_url_maps_to_web_fetch() {
    assert_eq!(
        normalize_tool("fetch_url", &CallerKind::Cursor),
        "web_fetch"
    );
}

#[test]
fn normalize_tool_cursor_spawn_agent_maps_to_spawn_agent() {
    assert_eq!(
        normalize_tool("spawn_agent", &CallerKind::Cursor),
        "spawn_agent"
    );
}

#[test]
fn normalize_tool_cursor_get_diagnostics_maps_to_diagnostics() {
    assert_eq!(
        normalize_tool("get_diagnostics", &CallerKind::Cursor),
        "diagnostics"
    );
}

#[test]
fn normalize_tool_cursor_move_file_maps_to_move_file() {
    assert_eq!(
        normalize_tool("move_file", &CallerKind::Cursor),
        "move_file"
    );
}

#[test]
fn normalize_tool_cursor_delete_file_maps_to_delete_file() {
    assert_eq!(
        normalize_tool("delete_file", &CallerKind::Cursor),
        "delete_file"
    );
}

#[test]
fn normalize_tool_cursor_create_dir_maps_to_create_dir() {
    assert_eq!(
        normalize_tool("create_dir", &CallerKind::Cursor),
        "create_dir"
    );
}

// ---------------------------------------------------------------------------
// normalize_tool — Windsurf mappings missing from integration_test.rs
// ---------------------------------------------------------------------------

#[test]
fn normalize_tool_windsurf_edit_file_maps_to_edit_file() {
    assert_eq!(
        normalize_tool("edit_file", &CallerKind::Windsurf),
        "edit_file"
    );
}

#[test]
fn normalize_tool_windsurf_find_files_maps_to_glob() {
    assert_eq!(
        normalize_tool("find_files", &CallerKind::Windsurf),
        "glob"
    );
}

#[test]
fn normalize_tool_windsurf_fetch_page_maps_to_web_fetch() {
    assert_eq!(
        normalize_tool("fetch_page", &CallerKind::Windsurf),
        "web_fetch"
    );
}

#[test]
fn normalize_tool_windsurf_spawn_agent_maps_to_spawn_agent() {
    assert_eq!(
        normalize_tool("spawn_agent", &CallerKind::Windsurf),
        "spawn_agent"
    );
}

#[test]
fn normalize_tool_windsurf_get_diagnostics_maps_to_diagnostics() {
    assert_eq!(
        normalize_tool("get_diagnostics", &CallerKind::Windsurf),
        "diagnostics"
    );
}

#[test]
fn normalize_tool_windsurf_move_file_maps_to_move_file() {
    assert_eq!(
        normalize_tool("move_file", &CallerKind::Windsurf),
        "move_file"
    );
}

#[test]
fn normalize_tool_windsurf_delete_file_maps_to_delete_file() {
    assert_eq!(
        normalize_tool("delete_file", &CallerKind::Windsurf),
        "delete_file"
    );
}

#[test]
fn normalize_tool_windsurf_create_directory_maps_to_create_dir() {
    assert_eq!(
        normalize_tool("create_directory", &CallerKind::Windsurf),
        "create_dir"
    );
}

// ---------------------------------------------------------------------------
// normalize_tool — Cline mappings missing from integration_test.rs
// ---------------------------------------------------------------------------

#[test]
fn normalize_tool_cline_search_maps_to_web_search() {
    assert_eq!(normalize_tool("search", &CallerKind::Cline), "web_search");
}

#[test]
fn normalize_tool_cline_fetch_maps_to_web_fetch() {
    assert_eq!(normalize_tool("fetch", &CallerKind::Cline), "web_fetch");
}

#[test]
fn normalize_tool_cline_rename_file_maps_to_move_file() {
    assert_eq!(
        normalize_tool("rename_file", &CallerKind::Cline),
        "move_file"
    );
}

#[test]
fn normalize_tool_cline_delete_file_maps_to_delete_file() {
    assert_eq!(
        normalize_tool("delete_file", &CallerKind::Cline),
        "delete_file"
    );
}

#[test]
fn normalize_tool_cline_create_directory_maps_to_create_dir() {
    assert_eq!(
        normalize_tool("create_directory", &CallerKind::Cline),
        "create_dir"
    );
}

#[test]
fn normalize_tool_cline_get_diagnostics_maps_to_diagnostics() {
    assert_eq!(
        normalize_tool("get_diagnostics", &CallerKind::Cline),
        "diagnostics"
    );
}

// ---------------------------------------------------------------------------
// normalize_tool — Amp mappings missing from integration_test.rs
// ---------------------------------------------------------------------------

#[test]
fn normalize_tool_amp_search_grep_maps_to_grep() {
    assert_eq!(normalize_tool("search.grep", &CallerKind::Amp), "grep");
}

#[test]
fn normalize_tool_amp_search_glob_maps_to_glob() {
    assert_eq!(normalize_tool("search.glob", &CallerKind::Amp), "glob");
}

#[test]
fn normalize_tool_amp_web_fetch_maps_to_web_fetch() {
    assert_eq!(normalize_tool("web.fetch", &CallerKind::Amp), "web_fetch");
}

#[test]
fn normalize_tool_amp_agent_spawn_maps_to_spawn_agent() {
    assert_eq!(
        normalize_tool("agent.spawn", &CallerKind::Amp),
        "spawn_agent"
    );
}

#[test]
fn normalize_tool_amp_lsp_diagnostics_maps_to_diagnostics() {
    assert_eq!(
        normalize_tool("lsp.diagnostics", &CallerKind::Amp),
        "diagnostics"
    );
}

#[test]
fn normalize_tool_amp_fs_move_maps_to_move_file() {
    assert_eq!(normalize_tool("fs.move", &CallerKind::Amp), "move_file");
}

#[test]
fn normalize_tool_amp_fs_delete_maps_to_delete_file() {
    assert_eq!(
        normalize_tool("fs.delete", &CallerKind::Amp),
        "delete_file"
    );
}

#[test]
fn normalize_tool_amp_fs_mkdir_maps_to_create_dir() {
    assert_eq!(normalize_tool("fs.mkdir", &CallerKind::Amp), "create_dir");
}

// ---------------------------------------------------------------------------
// HookResponse::modify with a non-object Value → empty input map
// ---------------------------------------------------------------------------

#[test]
fn hook_response_modify_with_string_value_produces_empty_input() {
    // A JSON string is not an object; as_object() returns None → unwrap_or_default → empty map.
    let resp = HookResponse::modify(serde_json::Value::String("not an object".to_string()));
    match resp {
        HookResponse::ModifyResponse(m) => {
            assert!(
                m.input.is_empty(),
                "expected empty input map, got: {:?}",
                m.input
            );
        }
        other => panic!("expected ModifyResponse, got: {:?}", other),
    }
}

#[test]
fn hook_response_modify_with_array_value_produces_empty_input() {
    let resp = HookResponse::modify(serde_json::json!([1, 2, 3]));
    match resp {
        HookResponse::ModifyResponse(m) => {
            assert!(m.input.is_empty());
        }
        other => panic!("expected ModifyResponse, got: {:?}", other),
    }
}

#[test]
fn hook_response_modify_with_number_value_produces_empty_input() {
    let resp = HookResponse::modify(serde_json::Value::Number(42.into()));
    match resp {
        HookResponse::ModifyResponse(m) => {
            assert!(m.input.is_empty());
        }
        other => panic!("expected ModifyResponse, got: {:?}", other),
    }
}

#[test]
fn hook_response_modify_with_bool_value_produces_empty_input() {
    let resp = HookResponse::modify(serde_json::Value::Bool(true));
    match resp {
        HookResponse::ModifyResponse(m) => {
            assert!(m.input.is_empty());
        }
        other => panic!("expected ModifyResponse, got: {:?}", other),
    }
}

#[test]
fn hook_response_modify_with_null_value_produces_empty_input() {
    let resp = HookResponse::modify(serde_json::Value::Null);
    match resp {
        HookResponse::ModifyResponse(m) => {
            assert!(m.input.is_empty());
        }
        other => panic!("expected ModifyResponse, got: {:?}", other),
    }
}

#[test]
fn hook_response_modify_with_object_value_preserves_fields() {
    let obj = serde_json::json!({"key": "value", "num": 42});
    let resp = HookResponse::modify(obj.clone());
    match resp {
        HookResponse::ModifyResponse(m) => {
            assert_eq!(m.input.len(), 2);
            assert_eq!(m.input["key"], "value");
            assert_eq!(m.input["num"], 42);
        }
        other => panic!("expected ModifyResponse, got: {:?}", other),
    }
}
