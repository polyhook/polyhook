package polyhook

import "sync"

// defaultLoader is the wasmLoader value at package init time; used by
// resetRuntime (in wasm_helper_test.go) to restore clean state between tests.
var defaultLoader = wasmLoader

// Exported only for testing.
var (
	// ResetRuntime tears down the singleton WASM runtime so the next call to
	// getRuntime() will re-initialise it. Does NOT restore wasmLoader — call
	// RestoreWasmLoader separately if needed.
	ResetRuntime = func() {
		runtimeOnce = sync.Once{}
		runtime_ = nil
		runtimeErr = nil
		mu.Lock()
		lastCaller = ""
		mu.Unlock()
	}

	// RestoreWasmLoader resets wasmLoader to the package-initialised default.
	RestoreWasmLoader = func() { wasmLoader = defaultLoader }

	// SetWasmLoader replaces the package-level wasmLoader function. The
	// supplied fn is called by getRuntime() when it initialises the singleton.
	SetWasmLoader = func(fn func() ([]byte, error)) { wasmLoader = fn }

	// SetTestParser installs a Go-level mock for the WASM parse export.
	// When set, ReadFrom calls this function instead of the WASM runtime.
	SetTestParser = func(fn func([]byte) ([]byte, error)) { testParser = fn }

	// SetTestSerializer installs a Go-level mock for the WASM serialize export.
	// When set, RespondTo calls this function instead of the WASM runtime.
	SetTestSerializer = func(fn func([]byte) ([]byte, error)) { testSerializer = fn }

	// ClearTestHooks removes both mocks so subsequent calls use the real WASM runtime.
	ClearTestHooks = func() { testParser = nil; testSerializer = nil }

	// SetWasmBytes sets the embedded-binary cache used by defaultWASMLoader.
	// The loader returns these bytes immediately without reading from disk.
	SetWasmBytes = func(b []byte) { wasmBytes = b }
)
