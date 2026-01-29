# Regression Test Tool (RTT1)

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

## Overview

RTT1 captures command output and exit codes as "golden" test results, then compares subsequent runs against these baselines to detect regressions. It's useful for:

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
git clone https://github.com/wrightmikea/rtt1.git
cd rtt1

# Build the project
cargo build --release

# The binary is at ./target/release/rtt1
```

## Quick Start

```bash
# 1. Create a test that captures the output of a command
rtt1 create -t data/my_test.db -c "echo hello world"

# 2. Run the test again to check for regressions
rtt1 run -p data/my_test.db

# 3. View the test results
rtt1 report -p data/my_test.db -v

# 4. If you modify the expected behavior, remove and recreate the test
rtt1 remove -p data/my_test.db
rtt1 create -t data/my_test.db -c "echo hello world"
```

## Commands

### create

Creates a new test by running a command and storing its output as the baseline.

```bash
rtt1 create -t <test_name> -c <command>

# Options:
#   -t, --test <name>      Name of the test (path to SQLite database file)
#   -c, --command <cmd>    Command to execute and capture

# Examples:
rtt1 create -t data/pwd_test.db -c "pwd"
rtt1 create -t data/version_test.db -c "git --version"
rtt1 c -t data/ls_test.db -c "ls -la"   # 'c' is an alias for 'create'
```

### run

Runs previously created tests and compares results against baselines.

```bash
rtt1 run -p <pattern>

# Options:
#   -p, --pattern <pat>    Pattern to match test names (supports glob patterns)
#   -n, --dry-run          Print what would be run without executing

# Examples:
rtt1 run -p data/pwd_test.db           # Run a specific test
rtt1 run -p "data/*.db"                # Run all tests in data/
rtt1 r -p data/pwd_test.db -n          # 'r' is alias; dry-run mode
```

### report

Reports on test results with configurable verbosity.

```bash
rtt1 report -p <pattern> [-v|-vv|-vvv]

# Options:
#   -p, --pattern <pat>    Pattern to match test names
#   -v                     Show test names
#   -vv                    Show test names and failure info
#   -vvv                   Show test names, failures, and differences

# Examples:
rtt1 report -p data/pwd_test.db        # Basic summary
rtt1 report -p "data/*.db" -v          # Show names
rtt1 p -p data/pwd_test.db -vvv        # 'p' is alias; full details
```

### remove

Removes tests and their associated result databases.

```bash
rtt1 remove -p <pattern>

# Options:
#   -p, --pattern <pat>    Pattern to match tests to remove

# Examples:
rtt1 remove -p data/old_test.db
rtt1 remove -p "data/temp_*.db"
```

### status

Starts a web server to monitor test results.

```bash
rtt1 status -p <pattern> [-l <port>]

# Options:
#   -p, --pattern <pat>           Pattern to match tests to monitor
#   -l, --localhost-port <port>   Port number (default: 4111)

# Examples:
rtt1 status -p "data/*.db"
rtt1 status -p "data/*.db" -l 8080
rtt1 s -p "data/*.db"              # 's' is alias for 'status'
```

Open http://localhost:4111 (or your chosen port) to view the status page.

## Architecture

The codebase follows a modular architecture:

```
src/
├── main.rs          # Entry point
├── lib.rs           # Module declarations
├── args.rs          # CLI argument parsing (clap)
├── builder.rs       # Test creation logic
├── command.rs       # Subcommand dispatch
├── config.rs        # Configuration management
├── db.rs            # Database operations with file locking
├── diff.rs          # Test result comparison
├── error.rs         # Error types (thiserror)
├── executor.rs      # Command execution (with DI support)
├── finder.rs        # Test discovery
├── process.rs       # Process execution
├── queries.rs       # SQL query building
├── runner.rs        # Test execution logic
├── sqlite.rs        # SQLite interface
├── status/          # Web status server (axum)
│   ├── mod.rs
│   ├── server.rs
│   ├── monitor.rs
│   └── views/
└── templates/       # SQL and HTML templates
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
git clone https://github.com/wrightmikea/rtt1.git
cd rtt1
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
cargo test integration_test_rtt1_help

# Run tests with output
cargo test -- --nocapture
```

## License

See LICENSE file for details.
