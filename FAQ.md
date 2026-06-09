# FAQ

## General

### What is polyhook?

polyhook is a multi-language SDK for AI coding agent hooks. It solves the problem that every AI coding tool (Claude Code, Cursor, Windsurf, Cline, Amp, Gemini CLI, …) sends hook events in a different stdin format and expects a different stdout format. polyhook detects which tool called your binary, parses the event into a single normalized struct, and serializes your response back in whatever format that tool expects.

You write `read()` → your logic → `respond()`. The binary works unchanged regardless of which tool invoked it.

### What problem does it solve?

Without polyhook you write a parser and serializer for each AI tool:

```
Claude Code  →  stdin: { "tool_name": "Bash", "tool_input": { "command": "..." } }
                stdout: { "decision": "block", "reason": "..." }

Cursor       →  stdin: { "type": "BeforeToolCall", "toolCall": { "name": "run_terminal_cmd", "args": {...} } }
                stdout: { "action": "deny", "message": "..." }
```

With polyhook you write the logic once. Detection and format translation happen inside the WASM core.

### Which AI tools are supported?

| Tool | Status |
|---|---|
| Claude Code | Supported |
| Cursor | Supported |
| Windsurf | Supported |
| Cline | Supported |
| Amp | Supported |
| Gemini CLI | Supported |
| Continue | In progress |
| Aider | In progress |
| GitHub Copilot | Planned |

### Which languages have SDKs?

TypeScript/JavaScript, Rust, Go, C#/.NET, Python. All expose the same two functions and wrap the same WASM binary, so behavior is identical across every runtime.

---

## Installation

### TypeScript / JavaScript

```bash
npm install @polyhook/sdk
```

### Rust

```bash
cargo add polyhook
```

### Go

```bash
go get github.com/tupe12334/polyhook/packages/sdk-go
```

### C# / .NET

```bash
dotnet add package Polyhook.Sdk
```

### Python

```bash
pip install polyhook
```

---

## Writing Hooks

### What does a minimal hook look like?

**TypeScript**
```typescript
import { read, respond, approve } from "@polyhook/sdk";

const event = await read();
await respond(approve());
```

**Python**
```python
import polyhook

event = polyhook.read()
polyhook.respond(polyhook.approve())
```

**Go**
```go
event, err := polyhook.Read()
polyhook.Respond(polyhook.Approve())
```

**Rust**
```rust
let event = polyhook::read()?;
polyhook::respond(&HookResponse::approve())?;
```

### What fields does `HookEvent` have?

```typescript
interface HookEvent {
  event:     "tool:before" | "tool:after" | "session:start" | "session:stop" | "agent:stop" | "notification";
  tool?:     string;                        // normalized tool name, e.g. "bash", "write_file"
  input?:    Record<string, unknown>;       // tool input arguments (tool:before only)
  output?:   Record<string, unknown>;       // tool output (tool:after only)
  sessionId: string;
  agentId?:  string;                        // present only inside sub-agent context
  caller:    "claude-code" | "cursor" | "windsurf" | "cline" | "amp" | "gemini-cli" | "unknown";
}
```

`tool` is only present for `tool:before` and `tool:after` events. `input` is only present for `tool:before`. `output` is only present for `tool:after`.

### What are the three response actions?

| Action | Effect |
|---|---|
| `approve` | Proceed with the operation unchanged. |
| `block` | Abort the operation; surface your message to the user. |
| `modify` | Execute the operation with a different input than the AI tool provided. |

```typescript
approve()
block("reason shown to user")
modify({ command: "echo 'safe version'" })  // replacement input for the tool
```

### Can I use `modify` on any tool?

`modify` is most useful on `tool:before` events. The replacement `input` object must be compatible with the tool being called — polyhook passes it through to the AI tool's executor. `modify` on non-tool events or `tool:after` events has no effect and falls back to `approve`.

### How do I block `rm -rf /` in bash?

**TypeScript**
```typescript
import { read, respond, block, approve } from "@polyhook/sdk";

const event = await read();
if (event.tool === "bash" && /rm\s+-rf\s+\//.test(event.input?.command ?? "")) {
  await respond(block("Refusing to delete from root"));
} else {
  await respond(approve());
}
```

**Python**
```python
import re, polyhook

event = polyhook.read()
if event.tool == "bash" and re.search(r"rm\s+-rf\s+/", (event.input or {}).get("command", "")):
    polyhook.respond(polyhook.block("Refusing to delete from root"))
else:
    polyhook.respond(polyhook.approve())
```

More examples in `packages/sdk-<lang>/examples/`.

### How do I react only to a specific AI tool?

Check `event.caller`:

```typescript
if (event.caller === "claude-code" && event.tool === "bash") { ... }
```

For tool-agnostic hooks, ignore `event.caller` entirely — that's the whole point of polyhook.

### What normalized tool names should I match against?

Common names:

| polyhook name | Meaning |
|---|---|
| `bash` | Shell command execution |
| `read_file` | Read a file |
| `write_file` | Write / create a file |
| `edit_file` | Edit / patch a file |
| `list_dir` | List directory contents |
| `grep` | Grep / search in files |
| `glob` | Find files by pattern |
| `web_search` | Web search |
| `web_fetch` | Fetch a URL |
| `spawn_agent` | Spawn a sub-agent |

Full mapping table: [docs/tool-names.md](docs/tool-names.md)

If polyhook encounters a vendor tool name not in its mapping table, it passes it through as-is.

---

## Caller Detection

### How does polyhook know which AI tool invoked my binary?

Detection runs in priority order inside `core`:

1. `POLYHOOK_CALLER` environment variable — explicit override, checked first
2. Tool-specific environment variables (`CLAUDE_CODE_VERSION`, `CURSOR_SESSION_ID`, `WINDSURF_SESSION_ID`, `CLINE_SESSION_ID`, `AMP_SESSION_ID`, `GEMINI_PROJECT_DIR`)
3. Stdin JSON shape heuristics — key names and structure
4. Falls back to `caller: "unknown"` with best-effort parsing

### Can I force-set the detected caller?

Yes. Set `POLYHOOK_CALLER` before invoking the hook binary:

```bash
POLYHOOK_CALLER=cursor my-hook-binary
```

Accepted values: `claude-code`, `cursor`, `windsurf`, `cline`, `amp`, `gemini-cli`.

### What happens when the caller is unknown?

polyhook attempts a best-effort parse and sets `caller: "unknown"`. The hook still runs — it does not crash. If your logic branches on `event.caller`, the `"unknown"` branch runs (or no branch runs if you don't handle it, which is fine for tool-agnostic hooks).

---

## Architecture

### Why is the core written in Rust compiled to WASM?

One implementation, every language. All detection, normalization, and serialization logic lives in `core` (Rust). Each language SDK is a thin shim that loads `polyhook.wasm` into a runtime, passes stdin bytes in, and reads the result out. No logic is re-implemented per language, so behavior is identical across every SDK and divergence is impossible.

### Does WASM add meaningful overhead?

No. Hook binaries are short-lived processes (invoked per hook event, not per request). WASM startup is in the single-digit millisecond range. The Rust SDK avoids WASM entirely by linking `core` natively.

### What WASM runtime does each SDK use?

| SDK | Runtime |
|---|---|
| TypeScript | wasm-bindgen (Node.js / browser) |
| Go | Wazero |
| C# | Wasmtime |
| Python | wasmtime-py |
| Rust | Native — no WASM overhead |

### Where does `polyhook.wasm` come from?

Built from `core/` via `make wasm`. The artifact is committed to the repo root and bundled into each SDK package. You don't need to build it yourself unless you're changing `core`.

### Can I use polyhook from a language not listed?

Yes. Any language with a WASM runtime can host `polyhook.wasm`. The raw WASM host API is documented in [BINDINGS.md](BINDINGS.md).

---

## Testing and Debugging

### How do I test my hook locally without an AI tool?

Pipe a mock stdin payload and inspect stdout:

```bash
echo '{"tool_name":"Bash","tool_input":{"command":"ls"},"hook_event_name":"PreToolUse","session_id":"test","agent_id":""}' | my-hook-binary
```

The exact stdin format depends on the tool you want to simulate. See [ARCHITECTURE.md](ARCHITECTURE.md) for per-tool event shapes, or set `POLYHOOK_CALLER` to skip detection and force a specific caller format.

### Is there a debug CLI?

A `polyhook check` CLI — which prints the detected caller and parsed event for a given stdin — is on the roadmap but not yet shipped.

---

## Contributing

### How do I add support for a new AI tool?

All changes go in `core/src/`:

1. `detect.rs` — add env var check and/or stdin shape heuristic
2. `tools.rs` — add vendor tool name → canonical name mappings
3. `events.rs` — add vendor event name → canonical event name mappings
4. `response.rs` — add `HookResponse` → vendor response serialization

Then:
```bash
make wasm
```

All language SDKs pick up the changes automatically. Add a fixture in `core/tests/fixtures/` and update `tools.toml`, `README.md`, and `docs/tool-names.md`. See [CONTRIBUTING.md](CONTRIBUTING.md) for the full checklist.

### How do I add a new language binding?

1. Create `packages/sdk-<lang>/`
2. Load `polyhook.wasm` with your language's WASM runtime
3. Expose `read()` and `respond()` wrapping the WASM `parse` and `serialize` exports
4. Follow the host API in [BINDINGS.md](BINDINGS.md)
5. Add an entry to the SDK table in [ARCHITECTURE.md](ARCHITECTURE.md)

### Where does the canonical type schema live?

`schema.json` at the repo root. All SDK types (`HookEvent`, `HookResponse`, `CallerKind`, etc.) are generated from it at build time. Editing a field here and rebuilding propagates the change to every language SDK simultaneously. Never hand-edit generated type files.

---

## Roadmap

### What's coming next?

See [ROADMAP.md](ROADMAP.md) for the full list. Near-term highlights:

- **Continue and Aider support** — detection heuristics and event mappings in progress
- **Ergonomic helpers** — `event.is_bash()`, `event.command()`, typed `input` fields per tool
- **`polyhook check` CLI** — debug tool: prints detected caller and parsed event for a given stdin
- **GitHub Copilot** — planned, blocked on hooks API availability

### When will Continue / Aider be supported?

Both are in progress. Detection heuristics and event mappings are being added to `core`. Follow the repo or watch the [ROADMAP.md](ROADMAP.md) for updates.

### Will there be a hook registry / package hub?

On the long-term roadmap: a registry where you can publish and install hook packages by name (`polyhook install no-root-delete`). Not yet implemented.
