# reg-rs (regress)

A command-line utility that creates, discovers, runs, and reports on regression tests.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
  - [create](#create)
  - [run](#run)
  - [report](#report)
  - [remove](#remove)
  - [status](#status)
- [Architecture](#architecture)
- [Development](#development)
- [License](#license)

## Overview

reg-rs captures command output and exit codes as "golden" test results, then compares subsequent runs against these baselines to detect regressions. It's useful for:

- Ensuring CLI tools maintain consistent behavior
- Detecting unexpected changes in command output
- Monitoring long-running test suites via web interface

### Features

- Create tests that capture command output and exit codes
- Run tests against matching patterns
- Report test results with varying verbosity
- Track and analyze differences between original and latest test runs
- Web-based status monitoring for long-running tests

## Installation

### Requirements

- Rust 2024 edition or later
- Cargo build tool

### Building from Source

```bash
# Clone the repository
git clone https://github.com/sw-cli-tools/reg-rs.git
cd reg-rs

# Build the project
cargo build --release

# The binary is at ./target/release/reg-rs
```

## Quick Start

**Note:** Tests must be stored in the `data/` directory with a `.tdb` extension.

```bash
# Create the data directory
mkdir -p data

# 1. Create a test that captures the output of a command
reg-rs create -t data/my_test.tdb -c "echo hello world"

# 2. Run the test again to check for regressions (use pattern matching)
reg-rs run -p my_test

# 3. View the test results
reg-rs report -p my_test -v

# 4. If you modify the expected behavior, remove and recreate the test
reg-rs remove -p my_test
reg-rs create -t data/my_test.tdb -c "echo hello world"
```

## Commands

### create

Creates a new test by running a command and storing its output as the baseline.

```bash
reg-rs create -t <test_name> -c <command>

# Options:
#   -t, --test <name>      Name of the test (path to SQLite database file)
#   -c, --command <cmd>    Command to execute and capture

# Examples:
reg-rs create -t data/pwd_test.tdb -c "pwd"
reg-rs create -t data/version_test.tdb -c "git --version"
reg-rs c -t data/ls_test.tdb -c "ls -la"   # 'c' is an alias for 'create'
```

### run

Runs previously created tests and compares results against baselines.

```bash
reg-rs run -p <pattern>

# Options:
#   -p, --pattern <pat>    Pattern to match test names (supports glob patterns)
#   -n, --dry-run          Print what would be run without executing

# Examples:
reg-rs run -p data/pwd_test.tdb           # Run a specific test
reg-rs run -p "data/*.tdb"                # Run all tests in data/
reg-rs r -p data/pwd_test.tdb -n          # 'r' is alias; dry-run mode
```

### report

Reports on test results with configurable verbosity.

```bash
reg-rs report -p <pattern> [-v|-vv|-vvv]

# Options:
#   -p, --pattern <pat>    Pattern to match test names
#   -v                     Show test names
#   -vv                    Show test names and failure info
#   -vvv                   Show test names, failures, and differences

# Examples:
reg-rs report -p data/pwd_test.tdb        # Basic summary
reg-rs report -p "data/*.tdb" -v          # Show names
reg-rs p -p data/pwd_test.tdb -vvv        # 'p' is alias; full details
```

### remove

Removes tests and their associated result databases.

```bash
reg-rs remove -p <pattern>

# Options:
#   -p, --pattern <pat>    Pattern to match tests to remove

# Examples:
reg-rs remove -p data/old_test.tdb
reg-rs remove -p "data/temp_*.tdb"
```

### status

Starts a web server to monitor test results.

```bash
reg-rs status -p <pattern> [-l <port>]

# Options:
#   -p, --pattern <pat>           Pattern to match tests to monitor
#   -l, --localhost-port <port>   Port number (default: 4111)

# Examples:
reg-rs status -p "data/*.tdb"
reg-rs status -p "data/*.tdb" -l 8080
reg-rs s -p "data/*.tdb"              # 's' is alias for 'status'
```

Open http://localhost:4111 (or your chosen port) to view the status page.

## Architecture

The codebase follows a modular architecture:

```
src/
|-- main.rs          # Entry point
|-- lib.rs           # Module declarations
|-- args.rs          # CLI argument parsing (clap)
|-- builder.rs       # Test creation logic
|-- command.rs       # Subcommand dispatch
|-- config.rs        # Configuration management
|-- db.rs            # Database operations with file locking
|-- diff.rs          # Test result comparison
|-- error.rs         # Error types (thiserror)
|-- executor.rs      # Command execution (with DI support)
|-- finder.rs        # Test discovery
|-- process.rs       # Process execution
|-- queries.rs       # SQL query building
|-- runner.rs        # Test execution logic
|-- sqlite.rs        # SQLite interface
|-- status/          # Web status server (axum)
|   |-- mod.rs
|   |-- server.rs
|   |-- monitor.rs
|   +-- views/
+-- templates/       # SQL and HTML templates
```

### Data Flow

1. **Create**: `main.rs` -> `builder.rs` -> `process.rs` -> `db.rs`
2. **Run**: `main.rs` -> `runner.rs` -> `process.rs` -> `diff.rs` -> `db.rs`
3. **Report**: `main.rs` -> `reporters/` -> `db.rs`
4. **Status**: `main.rs` -> `status/server.rs` -> `db.rs`

## Development

### Setup

```bash
# Clone and build
git clone https://github.com/sw-cli-tools/reg-rs.git
cd reg-rs
cargo build

# Run tests
cargo test

# Run with debug output
cargo run -- -d <subcommand>
```

### Code Quality

This project uses Rust 2024 edition with strict linting enabled (`#![deny(warnings, missing_docs)]`). Before committing:

```bash
cargo fmt        # Format code
cargo clippy     # Check for linting issues
cargo test       # Run all tests
cargo doc        # Generate documentation
```

### Testing

```bash
# Run all tests
cargo test

# Run a specific test
cargo test integration_test_reg_rs_help

# Run tests with output
cargo test -- --nocapture
```

## License

MIT License - Copyright (c) 2020-2026 Michael A. Wright

See LICENSE file for details.
