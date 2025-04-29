# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Test Commands
- Build: `cargo build`
- Run: `cargo run -- [FLAGS] [SUBCOMMAND]`
- Test all: `cargo test`
- Run single test: `cargo test integration_test_rtt1_help`
- Lint: `cargo clippy`
- Fix lints: `cargo clippy --fix`
- Documentation: `cargo doc`

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