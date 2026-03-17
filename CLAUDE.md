# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
reg-rs (pronounced "regress") is a CLI utility for creating, running, and managing regression tests. It captures command output and exit codes, then compares test runs to detect differences.

## Build/Test Commands
- Build: `cargo build`
- Run: `cargo run -- [FLAGS] [SUBCOMMAND]`
- Test all: `cargo test`
- Run single test: `cargo test integration_test_reg_rs_help`
- Lint: `cargo clippy --tests`
- Fix lints: `cargo clippy --fix`
- Format: `cargo fmt`
- Documentation: `cargo doc`

## Architecture
The codebase is split across 6 component workspaces, each with focused crates (max 7 modules per crate):

### Components
- **reg-rs** (binary) — Thin CLI dispatcher: `src/main.rs` only
- **reg-rs-core** — Types, constants, SQLite storage, .rgt format
  - `types` — Error types, constants, normalization, regression types
  - `store` — SQLite read/write, file locking, queries
  - `store-rgt` — .rgt TOML parser/writer, path utilities
- **reg-rs-engine** — Test execution and reporting
  - `exec` — Command executor, process capture, preprocessing
  - `runner` — Test runner, diff engine, dispatch (.rgt vs .tdb)
  - `report` — Report generation (summary, details, failures, passes)
- **reg-rs-status** — Real-time web monitoring
  - `status` — Axum web server, SSE, file watcher
  - `renderer` — HTML templates, diff formatting
- **reg-rs-cli** — Argument parsing, configuration, test discovery
  - `args` — Clap Args struct, Subcommands enum, version generation
  - `config` — Config struct (7 methods), CreateOptions, builder, logging, time
  - `discover` — Test file finder, data directory discovery
- **reg-rs-app** — Command handlers and AI integration
  - `commands` — create, run, report handlers + shared utils
  - `inspect` — show (rgt/tdb split), list, complete handlers
  - `modify` — remove, rebase, reset, migrate, status handlers
  - `ai` — Claude API command generation, failure analysis, context gathering

### Key Patterns
- All commands flow through: `main.rs` → `builder::build()` → component crate handler
- Two test formats: `.rgt` (TOML spec + `.out`/`.err` baselines, git-friendly) and `.tdb` (SQLite, legacy)
- `.rgt` takes precedence over `.tdb` when both exist for the same test stem
- `.tdb` files serve as runtime cache for `.rgt` tests (latest results, diffs) — gitignored
- Data directory auto-discovered: `$REG_RS_DATA_DIR` → `./work/reg-rs/` → cwd (if has .tdb/.rgt) → `~/.local/reg-rs/`
- `-p` pattern is optional on all subcommands (defaults to match all)
- `resolve_test_path()` in `reg-rs-commands/utils.rs` auto-places tests in data dir and appends `.rgt`
- Debug output via `log::debug!()`, enabled with `-d` flag (sets log level to debug via `env_logger`)
- Status monitoring uses Axum web framework (async with Tokio)
- Dependency injection via `CommandExecutor` trait in `reg-rs-exec` with `MockCommandExecutor` for tests
- Each crate exposes `pub` API; internal helpers stay private or `pub(crate)`
- External `.lock` files prevent SQLite file-lock conflicts (one per `.tdb` database)
- DB functions use `with_lock()` RAII wrapper for consistent lock/unlock pattern
- Custom `RegError` enum in `reg-rs-types` with `thiserror` — used consistently across all crates
- `RegressionType::from_code()` parses type codes; `display_label()` converts to human-readable labels
- Magic strings extracted as constants: `TDB_EXTENSION`, `LOCK_EXTENSION`, `FILE_WATCH_DEBOUNCE_SECS`, `REQUIRED_BLANK`
- `reg-rs-args/build.rs` generates version string with timestamp into `$OUT_DIR/generated.rs`
- Demo scripts (`demo/*.sh`) are tested via `cargo test` — reg-rs dogfoods itself
- Per-test metadata stored in `metadata_table` (key-value pairs) in each `.tdb` file — backward compatible
- `--preprocess` flag on `create` stores a shell command applied to stdout/stderr before diffing
- `--describe` flag on `create` uses Claude API to generate commands from natural language
- `--diff-mode` flag on `create` selects built-in normalization: `text` (default), `json` (sorts keys), or `lines-unordered` (sorts lines)
- Preprocess and diff-mode compose: preprocess runs first (external), then diff-mode normalizes (built-in)
- `--context` flag on `create` runs a command and includes output in AI prompt (requires `--describe`)
- `--desc`, `--expects`, `--flaky-note` flags on `create` store self-documenting test metadata, shown in failure reports
- `reg-rs-runner/dispatch.rs` dispatches `.rgt` vs `.tdb` tests: `.rgt` reads spec from TOML + baselines from `.out`/`.err`; `.tdb` reads from SQLite
- `reg-rs-runner/diff.rs` `process_differences_with_settings()` accepts preprocess/diff_mode as params (for `.rgt` tests)
- Shell aliases in `bin/source-rg.sh`: `rnrg` (run), `adrg` (add), `lsrg` (list), `shrg` (show), `uprg` (rebase), etc.

## Code Style Guidelines
- Formatting: Run `cargo fmt` before committing
- Imports: Group external crates first, then std imports
- Documentation: All public items require doc comments (`///`)
- Modules: Documented in lib.rs with `/// Module name` format
- Error handling: Use custom `RegError` type (`Result<T> = Result<T, RegError>`) with `?` operator
- Naming: snake_case for variables/functions, CamelCase for types
- Warnings: #![deny(warnings, missing_docs)] is enforced
- Debugging: Use `log::debug!()` for debug output (enabled by `-d` flag or `RUST_LOG=debug`)
- HTML tags in docs must be properly closed (e.g., `&lt;tag&gt;`)
- File size limit: < 500 lines (prefer 200-300); function limit: < 50 lines (prefer 10-30)
- TDD workflow: Red/Green/Refactor cycle; see `docs/process.md` for full pre-commit checklist