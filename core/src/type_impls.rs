use crate::types::{ApproveResponse, BlockResponse, CallerKind, HookResponse, ModifyResponse};

impl Default for CallerKind {
    fn default() -> Self {
        CallerKind::Unknown
    }
}

impl HookResponse {
    pub fn approve() -> Self {
        HookResponse::ApproveResponse(ApproveResponse {
            action: "approve".to_string(),
        })
    }

    pub fn block(msg: &str) -> Self {
        HookResponse::BlockResponse(BlockResponse {
            action: "block".to_string(),
            message: msg.to_owned(),
        })
    }

    /// `input` must be a JSON object; non-object values are silently treated as empty.
    pub fn modify(input: serde_json::Value) -> Self {
        HookResponse::ModifyResponse(ModifyResponse {
            action: "modify".to_string(),
            input: input.as_object().cloned().unwrap_or_default(),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::types::HookResponse;

    #[test]
    fn modify_with_object_preserves_fields() {
        let obj = serde_json::json!({"key": "value", "num": 42});
        let resp = HookResponse::modify(obj);
        match resp {
            HookResponse::ModifyResponse(m) => {
                assert_eq!(m.input.len(), 2);
                assert_eq!(m.input["key"], "value");
                assert_eq!(m.input["num"], 42);
            }
            other => panic!("expected ModifyResponse, got: {:?}", other),
        }
    }

    #[test]
    fn modify_with_string_produces_empty_input() {
        let resp = HookResponse::modify(serde_json::Value::String("not an object".into()));
        match resp {
            HookResponse::ModifyResponse(m) => assert!(m.input.is_empty()),
            other => panic!("expected ModifyResponse, got: {:?}", other),
        }
    }

    #[test]
    fn modify_with_array_produces_empty_input() {
        let resp = HookResponse::modify(serde_json::json!([1, 2, 3]));
        match resp {
            HookResponse::ModifyResponse(m) => assert!(m.input.is_empty()),
            other => panic!("expected ModifyResponse, got: {:?}", other),
        }
    }

    #[test]
    fn modify_with_null_produces_empty_input() {
        let resp = HookResponse::modify(serde_json::Value::Null);
        match resp {
            HookResponse::ModifyResponse(m) => assert!(m.input.is_empty()),
            other => panic!("expected ModifyResponse, got: {:?}", other),
        }
    }
}
