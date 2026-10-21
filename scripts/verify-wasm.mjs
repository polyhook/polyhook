#!/usr/bin/env node
// Smoke-tests a *compiled* polyhook.wasm binary end-to-end: feeds it a raw
// Claude Code PreToolUse payload and asserts the resulting block response
// uses `hookSpecificOutput.permissionDecision: "deny"`, not the legacy
// top-level `decision: "block"` (which terminates the whole Claude Code
// session instead of denying just the one tool call — see issue #53).
//
// core/src/wasm.rs already has a native `cargo test` covering this exact
// logic, and packages/sdk-ts's unit tests cover the SDK API — but both run
// against source compiled for the *host* target, never against the actual
// wasm32-unknown-unknown binary that gets uploaded as a build artifact and
// shipped in the npm/PyPI packages. Issue #53 was a real-world case of that
// gap: the source was correct, but the published .wasm artifact was stale.
// This script closes it by instantiating the real binary the same way the
// release workflow is about to publish.

import { readFile } from 'node:fs/promises'

const wasmPath = process.argv[2]
if (wasmPath === undefined) {
  console.error('usage: node scripts/verify-wasm.mjs <path-to-polyhook.wasm>')
  process.exit(1)
}

function writeBytes(memory, ptr, bytes) {
  new Uint8Array(memory.buffer).set(bytes, ptr)
}

function readLengthPrefixed(memory, ptr) {
  const view = new DataView(memory.buffer)
  const len = view.getInt32(ptr, true)
  return new Uint8Array(memory.buffer, ptr + 4, len)
}

function call(wasm, fnName, text) {
  const bytes = new TextEncoder().encode(text)
  const ptr = wasm.alloc(bytes.length)
  writeBytes(wasm.memory, ptr, bytes)
  const outPtr = wasm[fnName](ptr, bytes.length)
  const payload = readLengthPrefixed(wasm.memory, outPtr)
  const outLen = new DataView(wasm.memory.buffer).getInt32(outPtr, true)
  const text_ = new TextDecoder().decode(payload)
  wasm.dealloc(outPtr, 4 + outLen)
  wasm.dealloc(ptr, bytes.length)
  return text_
}

function fail(message) {
  console.error(`FAIL: ${message}`)
  process.exit(1)
}

const wasmBytes = await readFile(wasmPath)
const { instance } = await WebAssembly.instantiate(wasmBytes)
const wasm = instance.exports

// The raw wire format Claude Code actually sends for PreToolUse (see #53's repro).
const preToolUse = JSON.stringify({
  hook_event_name: 'PreToolUse',
  tool_name: 'Bash',
  tool_input: { command: 'git push --no-verify origin main' },
  session_id: 'verify-wasm',
})

const parsed = call(wasm, 'parse', preToolUse)
if (!parsed.includes('tool:before')) {
  fail(`parse() did not recognize PreToolUse as tool:before:\n${parsed}`)
}

const blockResponse = JSON.stringify({
  action: 'block',
  message: 'test message',
})
const serialized = call(wasm, 'serialize', blockResponse)
const output = JSON.parse(serialized)

if (output.hookSpecificOutput?.permissionDecision !== 'deny') {
  fail(
    `expected hookSpecificOutput.permissionDecision === "deny", got:\n${serialized}\n\n` +
      'This wasm would emit the legacy `decision: "block"` format for Claude Code, ' +
      'which terminates the whole session instead of denying just the one tool call.'
  )
}
if (output.decision !== undefined) {
  fail(`unexpected top-level "decision" field in:\n${serialized}`)
}

console.log(
  `OK: ${wasmPath} emits hookSpecificOutput.permissionDecision:"deny" for a Claude Code PreToolUse block`
)
