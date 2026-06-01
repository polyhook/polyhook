package polyhook

import "sync"

// Exported only for testing.
var (
	// ResetRuntime tears down the singleton WASM runtime so the next call to
	// getRuntime() will re-initialise it. This lets individual tests inject
	// their own wasmLoader and get a fresh runtime each time.
	ResetRuntime = func() {
		runtimeOnce = sync.Once{}
		runtime_ = nil
		runtimeErr = nil
		mu.Lock()
		lastCaller = ""
		mu.Unlock()
	}

	// SetWasmLoader replaces the package-level wasmLoader function. The
	// supplied fn is called by getRuntime() when it initialises the singleton.
	SetWasmLoader = func(fn func() ([]byte, error)) { wasmLoader = fn }
)
