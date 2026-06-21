# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Refreshed the embedded `polyhook.wasm` artifact bundled with the Go SDK.

## [0.1.11] - 2026-06-19

### Fixed

- Moved `schema.json` into the core crate so it is included in the published
  package and crates.io publishing no longer fails ([#38]).

## [0.1.10] - 2026-06-18

### Fixed

- Pointed the TypeScript SDK `repository` URL at `polyhook/polyhook` so npm
  provenance/publishing works correctly ([#37]).

## [0.1.9] - 2026-06-18

### Fixed

- Reverted the TypeScript SDK package rename to restore the previous package
  name and unblock npm publishing ([#36]).

## [0.1.8] - 2026-06-18

### Fixed

- Infer `tool:before` / `tool:after` events when the vendor event field is
  missing from the hook envelope ([#34]).

## [0.1.7] - 2026-06-15

### Added

- Hermes Agent hook support.

### Changed

- Renamed the TypeScript SDK package to `polyhook` ([#21]).
- Regenerated SDK outputs via the `make` targets and preserved the generated
  TypeScript `HookResponse` type.

## [0.1.6] - 2026-06-10

### Added

- GitHub Pages landing page.
- `FAQ.md` documentation.

### Fixed

- Pointed `build.rs` at the workspace-root `schema.json` to eliminate the
  duplicate schema file ([#16]).

## [0.1.5] - 2026-06-05

### Added

- Gemini CLI hook support ([#10]).
- Project logo asset.

### Fixed

- Use `hookSpecificOutput` deny for Claude Code `PreToolUse` blocks ([#12]).
- Corrected logo alignment in the README.

### Changed

- Colocated Rust tests into `_tests.rs` files and enabled `clippy::all` plus
  warnings-as-errors workspace-wide.

## [0.1.4] - 2026-06-02

### Fixed

- Detect `hook_event_name` from the real Claude Code runtime ([#6]).

## [0.1.3] - 2026-06-02

### Changed

- Migrated the TypeScript SDK build to Vite with dual ESM + CJS output ([#4]),
  and updated the README accordingly.

## [0.1.0] - 2026-06-02

Initial documented history. This entry rolls up the initial public release and
the early publishing/CI hardening that followed across v0.1.0, v0.1.1 and
v0.1.2.

### Added

- Initial release of Polyhook: a Rust core library with SDKs for TypeScript,
  Go, Python and .NET, plus the JSON schema for hook type definitions, the
  workspace `Cargo.toml`, Makefile, and per-package READMEs.
- GitHub Actions CI workflows with coverage enforcement across all SDKs.
- MIT license, cspell spell checking, and a pre-push git hook.

### Fixed

- Registry publishing reliability: OIDC trusted publishing for npm, retry and
  backoff for crates.io indexing races, embedded and refreshed `polyhook.wasm`
  for the Go SDK, and assorted module-path and metadata corrections.

[Unreleased]: https://github.com/polyhook/polyhook/compare/v0.1.11...HEAD
[0.1.11]: https://github.com/polyhook/polyhook/compare/v0.1.10...v0.1.11
[0.1.10]: https://github.com/polyhook/polyhook/compare/v0.1.9...v0.1.10
[0.1.9]: https://github.com/polyhook/polyhook/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/polyhook/polyhook/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/polyhook/polyhook/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/polyhook/polyhook/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/polyhook/polyhook/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/polyhook/polyhook/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/polyhook/polyhook/compare/v0.1.2...v0.1.3
[0.1.0]: https://github.com/polyhook/polyhook/releases/tag/v0.1.0
[#4]: https://github.com/polyhook/polyhook/pull/4
[#6]: https://github.com/polyhook/polyhook/pull/6
[#10]: https://github.com/polyhook/polyhook/pull/10
[#12]: https://github.com/polyhook/polyhook/pull/12
[#16]: https://github.com/polyhook/polyhook/pull/16
[#21]: https://github.com/polyhook/polyhook/pull/21
[#34]: https://github.com/polyhook/polyhook/pull/34
[#36]: https://github.com/polyhook/polyhook/pull/36
[#37]: https://github.com/polyhook/polyhook/pull/37
[#38]: https://github.com/polyhook/polyhook/pull/38
