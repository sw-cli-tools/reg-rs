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
The codebase follows a modular architecture with clear separation of concerns:

- **Main Entry**: `src/main.rs` + `src/lib.rs` - Entry point and module declarations
- **Command Processing**: `src/command.rs` + `src/builder.rs` - Handles subcommands (create, run, report, remove, status, analyze)
- **Test Execution**: `src/runner.rs` + `src/process.rs` - Runs tests and captures output
- **Preprocessing**: `src/preprocess.rs` - Output normalization before diffing (pipes through shell commands)
- **AI Integration**: `src/ai.rs` - Natural language test creation via Claude API (`--describe` flag)
- **Database Layer**: `src/db.rs` + `src/sqlite.rs` + `src/queries.rs` - SQLite storage for test results
- **Reporting**: `src/reporters/*.rs` - Different report formats (summary, details, passes, failures, differences)
- **Status Server**: `src/status/*.rs` - Web-based monitoring server (port 4111)
- **Templates**: `src/templates/*.rs` - SQL and HTML template generation

## Key Patterns
- All commands flow through: `main.rs` → `builder::build()` → `command::{action}()` → specific modules
- Test results stored in SQLite databases (one `.tdb` per test) in `~/.local/reg-rs/` by default
- Data directory overridable via `REG_RS_DATA_DIR` env var (used by tests and demos: `./work/reg-rs/`)
- `resolve_test_path()` in `command.rs` auto-places tests in data dir and appends `.tdb`
- Debug output via `log::debug!()`, enabled with `-d` flag (sets log level to debug via `env_logger`)
- Status monitoring uses Axum web framework (async with Tokio)
- Dependency injection via `CommandExecutor` trait (`executor.rs`) with `MockCommandExecutor` for tests
- Internal functions use `pub(crate)` visibility; only types/functions needed by `main.rs` are `pub`
- External `.lock` files prevent SQLite file-lock conflicts (one per `.tdb` database)
- DB functions use `with_lock()` RAII wrapper for consistent lock/unlock pattern
- Custom `RegError` enum (`error.rs`) with `thiserror` — used consistently across all modules
- `RegressionType::from_code()` parses type codes; `display_label()` converts to human-readable labels
- Magic strings extracted as constants: `TDB_EXTENSION`, `LOCK_EXTENSION`, `FILE_WATCH_DEBOUNCE_SECS`, `REQUIRED_BLANK`
- `build.rs` generates version string with timestamp into `$OUT_DIR/generated.rs`
- Demo scripts (`demo/*.sh`) are tested via `cargo test` — reg-rs dogfoods itself
- Per-test metadata stored in `metadata_table` (key-value pairs) in each `.tdb` file — backward compatible
- `--preprocess` flag on `create` stores a shell command applied to stdout/stderr before diffing
- `--describe` flag on `create` uses Claude API to generate commands from natural language
- `--diff-mode` flag on `create` selects built-in normalization: `text` (default), `json` (sorts keys), or `lines-unordered` (sorts lines)
- Preprocess and diff-mode compose: preprocess runs first (external), then diff-mode normalizes (built-in)
- `--context` flag on `create` runs a command and includes output in AI prompt (requires `--describe`)
- `--desc`, `--expects`, `--flaky-note` flags on `create` store self-documenting test metadata, shown in failure reports

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