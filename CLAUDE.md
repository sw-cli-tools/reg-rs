# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
RTT1 (Regression Test Tool) is a CLI utility for creating, running, and managing regression tests. It captures command output and exit codes, then compares test runs to detect differences.

## Build/Test Commands
- Build: `cargo build`
- Run: `cargo run -- [FLAGS] [SUBCOMMAND]`
- Test all: `cargo test`
- Run single test: `cargo test integration_test_rtt1_help`
- Lint: `cargo clippy`
- Fix lints: `cargo clippy --fix`
- Format: `cargo fmt`
- Documentation: `cargo doc`

## Architecture
The codebase follows a modular architecture with clear separation of concerns:

- **Main Entry**: `src/main.rs` + `src/lib.rs` - Entry point and module declarations
- **Command Processing**: `src/command.rs` + `src/builder.rs` - Handles subcommands (create, run, report, remove, status)
- **Test Execution**: `src/runner.rs` + `src/process.rs` - Runs tests and captures output
- **Database Layer**: `src/db.rs` + `src/sqlite.rs` + `src/queries.rs` - SQLite storage for test results
- **Reporting**: `src/reporters/*.rs` - Different report formats (summary, details, passes, failures, differences)
- **Status Server**: `src/status/*.rs` - Web-based monitoring server (port 4111)
- **Templates**: `src/templates/*.rs` - SQL and HTML template generation

## Key Patterns
- All commands flow through: `main.rs` → `builder::build()` → `command::{action}()` → specific modules
- Test results stored in SQLite databases (one per test)
- Conditional debug output via `md!()` macro when `-d` flag is used
- Status monitoring uses Gotham web framework

## Code Style Guidelines
- Formatting: Run `cargo fmt` before committing
- Imports: Group external crates first, then std imports
- Documentation: All public items require doc comments (`///`)
- Modules: Documented in lib.rs with `/// Module name` format
- Error handling: Use Result<T, Box<dyn std::error::Error>> with ? operator
- Naming: snake_case for variables/functions, CamelCase for types
- Warnings: #![deny(warnings, missing_docs)] is enforced
- Debugging: Use the `md!()` macro for conditional debug output
- HTML tags in docs must be properly closed (e.g., `&lt;tag&gt;`)