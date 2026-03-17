# reg-rs (regress)

A command-line utility that creates, discovers, runs, and reports on regression tests.

## Table of Contents

- [Overview](#overview)
- [Demos](#demos)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Handling Regressions](#handling-regressions)
- [Test Formats](#test-formats)
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
- Run tests with configurable verbosity (`-v`, `-vv`) or quiet mode (`-q`)
- Rebase baselines when output changes intentionally
- Shell aliases for quick access (`rnrg`, `lsrg`, `shrg`, etc.)
- AI-powered test creation from natural language descriptions
- Web-based status monitoring with live SSE updates
- Parallel test execution

## Demos

### Basic Workflow

![Basic Workflow](demo/basic.gif)

*Create, run, and report on a regression test. Generate with `vhs demo/basic.tape`.*

### Regression Detection

![Regression Detection](demo/workflow.gif)

*Detect regressions when command output changes. Generate with `vhs demo/workflow.tape`.*

### Dogfooding

![Dogfood Demo](demo/dogfood.gif)

*reg-rs tests its own CLI help output for regressions. Generate with `vhs demo/dogfood.tape`.*

### Web Status Dashboard

![Landing Page](images/landing.png?ts=1773786467090)

*Summary landing page showing 65 tests across all projects (reg-rs, favicon, pjmai, rank-wav, cor24) with pass/fail/pending counts.*

![Status Details](images/status-details.png?ts=1773786467090)

*Detailed status view with per-test results, character-level diff highlighting for failures, and live SSE updates.*

## Installation

### Requirements

- Rust 2024 edition or later
- Cargo build tool

### Building from Source

```bash
git clone https://github.com/sw-cli-tools/reg-rs.git
cd reg-rs
cargo build --release

# Install the binary
cargo install --path .
```

### Shell Aliases (for interactive use)

The shell aliases are the recommended way to use reg-rs interactively. Add to your `.bashrc` or `.zshrc`:

```bash
source /path/to/reg-rs/bin/source-rg.sh
```

This gives you short commands with tab-completion:

| Alias | Action | Example |
|-------|--------|---------|
| `adrg` | Add/create a test | `adrg my_test 'echo hi'` |
| `rnrg` | Run tests | `rnrg` or `rnrg my_test` |
| `lsrg` | List tests with status | `lsrg` |
| `shrg` | Show test details | `shrg my_test -v` |
| `uprg` | Rebase — accept new baseline | `uprg my_test` |
| `rsrg` | Reset test results | `rsrg my_test` |
| `rmrg` | Remove test | `rmrg old_test` |
| `strg` | Start status server | `strg` |
| `hlrg` | Show alias help | `hlrg` |

## Quick Start

Tests are stored as `.rgt` files (TOML specs) with `.out`/`.err` baselines — all git-friendly text.
reg-rs auto-discovers tests by checking (in order):

1. `$REG_RS_DATA_DIR` (if set)
2. `./work/reg-rs/` (if it exists)
3. Current directory (if it contains `.tdb` or `.rgt` files)
4. `~/.local/reg-rs/` (default)

```bash
# 1. Create a test
adrg hello 'echo hello world'

# 2. Run it
rnrg hello

# 3. Check results
lsrg                              # quick status overview
shrg hello -v                     # detailed view with baseline

# 4. Run all tests
rnrg

# 5. Clean up
rmrg hello
```

## Usage

### Creating tests

```bash
# Basic: capture command output as baseline
adrg my_test 'echo hello'

# With metadata
adrg my_test 'echo hello' --desc "Tests echo output" --timeout 10

# With preprocessing (normalize output before diffing)
adrg api_test 'curl -s localhost/api' -P "jq --sort-keys"

# With diff mode (built-in normalization)
adrg json_test 'myapp --json' -M json

# AI-generated command (requires ANTHROPIC_API_KEY)
reg-rs create -t ls_test -D "list files sorted by size"
```

### Running tests

```bash
rnrg                               # run all tests (summary line)
rnrg my_test                       # run matching tests
rnrg my_test -v                    # show failure details
rnrg my_test -vv                   # show failure details with full diffs
rnrg -q                            # quiet mode: exit code only (0=pass, 1=fail)
rnrg my_test --parallel            # run in parallel
```

### Viewing results

```bash
# Quick status
lsrg                               # all tests
lsrg my_test                       # matching tests

# Detailed view
shrg my_test                        # command, metadata, status
shrg my_test -v                     # + baseline stdout/stderr
shrg my_test -vv                    # + latest results and diffs
```

### Managing baselines

```bash
# Accept latest output as new baseline
uprg my_test

# Clear latest results (mark as pending, keep baselines)
rsrg my_test

# Remove test entirely
rmrg my_test
```

### Web dashboard

```bash
strg                                # start on default port 4740
strg my_test                        # filtered to matching tests
```

Open http://localhost:4740 to view live results with auto-updating via SSE.

## Handling Regressions

```bash
# Run tests — one fails
rnrg
# version: FAIL

# See what changed
shrg version -vv
# Shows: expected "1.0.0", got "1.1.0"

# If the change is intentional, accept the new baseline
uprg version

# If the change is a bug, fix the code and re-run
rnrg version
# version: PASS
```

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

Tests stored as SQLite databases. Use `mgrg` (or `reg-rs migrate`) to convert to `.rgt` format.

### Setting up regression tests for a project

```bash
cd my-project
mkdir -p work/reg-rs

# Create tests (reg-rs auto-discovers work/reg-rs/)
adrg version 'myapp --version'
adrg help 'myapp --help'
adrg basic 'myapp process input.txt'

# Run all tests
rnrg

# Track in git
echo "*.tdb" >> .gitignore
echo "*.lock" >> .gitignore
git add work/reg-rs/*.rgt work/reg-rs/*.out work/reg-rs/*.err .gitignore
git commit -m "add regression tests"
```

## Full Command Reference

The `reg-rs` CLI is designed for AI coding agents and shell scripts — explicit flags, structured output, and meaningful exit codes. For the full reference, see [docs/commands.md](docs/commands.md).

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

1. **Create**: `main.rs` -> `command.rs` -> `runner.rs` -> `process.rs` -> `rgt.rs`
2. **Run**: `main.rs` -> `command.rs` -> `runner.rs` -> `process.rs` -> `diff.rs` -> `db.rs`
3. **Report**: `main.rs` -> `command.rs` -> `reporters/` -> `db.rs`
4. **Status**: `main.rs` -> `command.rs` -> `status/server.rs` -> `db.rs`

## Development

### Setup

```bash
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
cargo test                                    # Run all tests
cargo test integration_test_reg_rs_help       # Run a specific test
cargo test -- --nocapture                     # Run tests with output
```

### Self-Testing (Dogfooding)

reg-rs tests its own CLI output for regressions. The demo scripts are run automatically as part of `cargo test`, so dogfooding is built into the routine test suite.

```bash
# cargo test runs all demo scripts (dogfood, basic workflow, regression detection)
cargo test

# Or run a demo script standalone
bash demo/dogfood.sh
bash demo/test_basic.sh
bash demo/test_workflow.sh
```

After making CLI changes, run `cargo test` to check if any help text changed unexpectedly.

## Known Gaps

See [docs/gaps.md](docs/gaps.md) for known limitations and proposed fixes, including:

- Pattern matching is substring-only (not regex/glob)

## License

MIT License - Copyright (c) 2020-2026 Michael A. Wright

See LICENSE file for details.
