# reg-rs (regress)

A command-line utility that creates, discovers, runs, and reports on regression tests.

## Table of Contents

- [Overview](#overview)
- [Demos](#demos)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
  - [create](#create)
  - [run](#run)
  - [list](#list)
  - [show](#show)
  - [report](#report)
  - [rebase](#rebase)
  - [migrate](#migrate)
  - [reset](#reset)
  - [remove](#remove)
  - [status](#status)
- [Test Formats](#test-formats)
- [Shell Aliases](#shell-aliases)
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
- Git-friendly `.rgt` text format (TOML spec + `.out`/`.err` baselines)
- Run tests against matching patterns (sequential or parallel)
- List, show, and report test results with varying verbosity
- Rebase baselines when output changes intentionally
- Migrate existing `.tdb` tests to `.rgt` format
- Shell aliases for quick access (`rnrg`, `lsrg`, `shrg`, etc.)
- AI-powered test creation from natural language descriptions
- Web-based status monitoring with live SSE updates

## Demos

### Basic Workflow

![Basic Workflow](demo/basic.gif)

*Create, run, report, and remove a regression test. Generate with `vhs demo/basic.tape`.*

### Regression Detection

![Regression Detection](demo/workflow.gif)

*Detect regressions when command output changes. Generate with `vhs demo/workflow.tape`.*

### Dogfooding

![Dogfood Demo](demo/dogfood.gif)

*reg-rs tests its own CLI help output for regressions. Generate with `vhs demo/dogfood.tape`.*

### Web Status Dashboard

![Web Status Dashboard](images/web-screen.png?ts=1773373732623)

*Live status dashboard showing 21 tests across multiple suites — failures with character-level diff highlighting, passing tests, and pending tests. Updates in real time via SSE.*

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

Tests are stored as `.rgt` files (TOML specs) with `.out`/`.err` baselines — all git-friendly text.
reg-rs auto-discovers tests by checking (in order):

1. `$REG_RS_DATA_DIR` (if set)
2. `./work/reg-rs/` (if it exists)
3. Current directory (if it contains `.tdb` or `.rgt` files)
4. `~/.local/reg-rs/` (default)

The `-p` pattern flag is optional — omit it to run all tests.

```bash
# 1. Create a test that captures the output of a command
reg-rs create -t my_test -c "echo hello world"

# 2. Run all tests (auto-discovers data dir, matches all)
reg-rs run

# 3. Run specific tests by pattern
reg-rs run -p my_test

# 4. View test results
reg-rs list                    # quick status overview
reg-rs show -p my_test -v      # detailed view with baseline
reg-rs report -v               # formal report

# 5. If output changes intentionally, accept the new baseline
reg-rs rebase -p my_test

# 6. Migrate existing .tdb tests to git-friendly .rgt format
reg-rs migrate
```

## Commands

### create

Creates a new test by running a command and storing its output as the baseline.

```bash
reg-rs create -t <test_name> -c <command>

# Options:
#   -t, --test <name>      Test name (stored as name.tdb in data dir)
#   -c, --command <cmd>    Command to execute and capture

# Examples:
reg-rs create -t pwd_test -c "pwd"
reg-rs create -t version_test -c "git --version"
reg-rs c -t ls_test -c "ls -la"   # 'c' is an alias for 'create'
```

### run

Runs previously created tests and compares results against baselines.

```bash
reg-rs run -p <pattern>

# Options:
#   -p, --pattern <pat>    Pattern to match test names (supports glob patterns)
#   -n, --dry-run          Print what would be run without executing

# Examples:
reg-rs run -p pwd_test                    # Run a specific test
reg-rs run -p test                        # Run all matching tests
reg-rs r -p pwd_test -n                   # 'r' is alias; dry-run mode
```

### list

Lists tests with their name, command, and status (PASS/FAIL/pending).

```bash
reg-rs list                           # list all tests
reg-rs list -p my_test                # list matching tests
reg-rs l -p test                      # 'l' is alias
```

### show

Shows detailed test information including command, metadata, baselines, and diffs.

```bash
reg-rs show -p my_test                # command and metadata
reg-rs show -p my_test -v             # also baseline output
reg-rs show -p my_test -vv            # also latest results and diffs
reg-rs w -p test                      # 'w' is alias
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
reg-rs report -p pwd_test                 # Basic summary
reg-rs report -p test -v                  # Show names
reg-rs p -p pwd_test -vvv                 # 'p' is alias; full details
```

### remove

Removes tests and their associated result databases.

```bash
reg-rs remove -p <pattern>

# Options:
#   -p, --pattern <pat>    Pattern to match tests to remove

# Examples:
reg-rs remove -p old_test
reg-rs remove -p temp_
```

### rebase

Accepts the latest test output as the new expected baseline.

```bash
reg-rs rebase -p my_test              # accept latest output
reg-rs u -p version                   # 'u' is alias (update)

# For .rgt tests: updates .out/.err files
# For .tdb tests: replaces original results with latest
```

### migrate

Converts existing `.tdb` tests to `.rgt` text format.

```bash
reg-rs migrate                        # migrate all tests
reg-rs migrate -p my_test             # migrate matching tests
reg-rs m -p old_test                  # 'm' is alias
```

### reset

Clears latest run results from the `.tdb` cache, keeping baselines intact.

```bash
reg-rs reset -p my_test               # reset matching tests
reg-rs reset                          # reset all tests
```

### status

Starts a web server to monitor test results.

```bash
reg-rs status -p <pattern> [-l <port>]

# Options:
#   -p, --pattern <pat>           Pattern to match tests to monitor
#   -l, --localhost-port <port>   Port number (default: 4740)

# Examples:
reg-rs status -p test
reg-rs status -p test -l 8080
reg-rs s -p test                      # 's' is alias for 'status'
```

Open http://localhost:4740 (or your chosen port) to view the status page.

## Test Formats

### .rgt Format (recommended)

Tests are defined as TOML `.rgt` files with companion `.out`/`.err` baselines — all git-friendly text:

| File | Purpose | Git tracked |
|------|---------|-------------|
| `.rgt` | Test spec (command, timeout, metadata) | Yes |
| `.out` | Expected stdout baseline | Yes |
| `.err` | Expected stderr (absent if empty) | Yes |
| `.tdb` | Runtime cache (latest results, diffs) | No |

Example `.rgt` file:

```toml
command = "echo hello"
timeout = 10
exit_code = 0
desc = "Hello world output"
```

Add to `.gitignore`:

```
*.tdb
*.lock
```

### .tdb Format (legacy)

Tests stored as SQLite databases. Use `reg-rs migrate` to convert to `.rgt` format.

## Shell Aliases

Source `bin/source-rg.sh` in your shell for quick access:

```bash
source /path/to/reg-rs/bin/source-rg.sh
```

| Alias | Action |
|-------|--------|
| `rnrg [pattern]` | Run tests |
| `adrg <name> '<cmd>'` | Add/create a test |
| `lsrg [pattern]` | List tests with status |
| `shrg <name> [-v]` | Show test details |
| `uprg <pattern>` | Rebase — accept latest as baseline |
| `rsrg <pattern>` | Reset test results |
| `rmrg <pattern>` | Remove test files |
| `mgrg [pattern]` | Migrate .tdb to .rgt |
| `strg [pattern]` | Start status server |
| `hlrg` | Show alias help |

Tab completion for test names is included (zsh and bash).

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
|-- finder.rs        # Test discovery (.rgt and .tdb)
|-- process.rs       # Process execution
|-- queries.rs       # SQL query building
|-- rgt.rs           # .rgt TOML format parsing and baseline I/O
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

1. **Create**: `main.rs` -> `command.rs` -> `runner.rs` -> `process.rs` -> `db.rs`
2. **Run**: `main.rs` -> `command.rs` -> `runner.rs` -> `process.rs` -> `diff.rs` -> `db.rs`
3. **Report**: `main.rs` -> `command.rs` -> `reporters/` -> `db.rs`
4. **Status**: `main.rs` -> `command.rs` -> `status/server.rs` -> `db.rs`

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

### Self-Testing (Dogfooding)

reg-rs tests its own CLI output for regressions. The demo scripts are run automatically as part of `cargo test`, so dogfooding is built into the routine test suite.

```bash
# cargo test runs all demo scripts (dogfood, basic workflow, regression detection)
cargo test

# Or run a demo script standalone (builds release, uses ./work/reg-rs/ for data)
bash demo/dogfood.sh
bash demo/test_basic.sh
bash demo/test_workflow.sh
```

The demo scripts accept `REG_RS_BIN` to use a specific binary (integration tests use the debug build automatically).

After making CLI changes, run `cargo test` to check if any help text changed unexpectedly. If the change was intentional, re-create the baselines with `demo/dogfood.sh`.

## Known Gaps

See [docs/gaps.md](docs/gaps.md) for known limitations and proposed fixes, including:

- Pattern matching is substring-only (not regex/glob)

## License

MIT License - Copyright (c) 2020-2026 Michael A. Wright

See LICENSE file for details.
