package polyhook_test

// wasm_helper_test.go provides a passthrough WAT-based WASM shim used by all
// tests that exercise the WASM glue code (ReadFrom, RespondTo, wasmCall, …).
//
// The WAT module below is a minimal stand-in for polyhook.wasm:
//   - alloc / dealloc  – bump allocator, dealloc is a no-op
//   - parse / serialize – both just echo the input bytes wrapped in a 4-byte
//     LE length prefix, which is exactly what wasmReadLengthPrefixed expects.
//
// wazero accepts WAT text directly when the byte slice starts with '(' rather
// than the WASM magic bytes, so we simply hand the WAT source to wasmLoader.

import polyhook "github.com/polyhook/polyhook-go"

const passthroughWAT = `(module
  (memory (export "memory") 2)
  (global $bump (mut i32) (i32.const 4096))

  (func (export "alloc") (param $len i32) (result i32)
    (local $ptr i32)
    global.get $bump
    local.tee $ptr
    local.get $len
    i32.add
    global.set $bump
    local.get $ptr
  )

  (func (export "dealloc") (param $ptr i32) (param $len i32))

  (func $wrap (param $ptr i32) (param $len i32) (result i32)
    (local $out i32)
    global.get $bump
    local.tee $out
    local.get $len
    i32.const 4
    i32.add
    i32.add
    global.set $bump
    local.get $out
    local.get $len
    i32.store
    local.get $out
    i32.const 4
    i32.add
    local.get $ptr
    local.get $len
    memory.copy
    local.get $out
  )

  (func (export "parse") (param $ptr i32) (param $len i32) (result i32)
    local.get $ptr
    local.get $len
    call $wrap
  )

  (func (export "serialize") (param $ptr i32) (param $len i32) (result i32)
    local.get $ptr
    local.get $len
    call $wrap
  )
)`

// usePassthroughWASM installs the passthrough WAT shim as the wasmLoader and
// resets the runtime singleton. Call resetRuntime() in a defer to restore
// state after the test.
func usePassthroughWASM() {
	polyhook.SetWasmLoader(func() ([]byte, error) {
		return []byte(passthroughWAT), nil
	})
	polyhook.ResetRuntime()
}

// resetRuntime tears down the runtime singleton so subsequent tests start
// fresh.
func resetRuntime() {
	polyhook.ResetRuntime()
}
