//! Test-only guard for the agent environment variables that caller detection
//! reads. temp_env runs every call under one global lock, so a test that
//! parses an event inside `with_clean_env` never races a test that sets one of
//! these variables, and never sees the developer's own agent environment.

pub(crate) const AGENT_ENV_VARS: &[&str] = &[
    "POLYHOOK_CALLER",
    "CLAUDE_CODE_VERSION",
    "CLAUDE_PROJECT_DIR",
    "CURSOR_SESSION_ID",
    "WINDSURF_SESSION_ID",
    "CLINE_SESSION_ID",
    "AMP_SESSION_ID",
    "GEMINI_PROJECT_DIR",
];

pub(crate) fn with_clean_env<R>(f: impl FnOnce() -> R) -> R {
    let vars: Vec<(&str, Option<&str>)> = AGENT_ENV_VARS.iter().map(|k| (*k, None)).collect();
    temp_env::with_vars(vars, f)
}
