# Development Process

## Overview

This document describes the development workflow for the reg-rs project.

## Core Principles

### Test-Driven Development (TDD)

**Red/Green/Refactor Cycle**:

```
RED: Write failing test -> GREEN: Make it pass -> REFACTOR: Improve code -> Repeat
```

### Pre-Commit Quality Gates

All changes must pass before every commit. No exceptions, no deferrals, no disabling checks.

**Pre-Commit Sequence**:

```bash
# 1. Run tests (including dogfood/demo script tests)
cargo test

# 2. Fix linting (zero warnings, never suppress with #[allow(...)])
cargo clippy --tests

# 3. Format code
cargo fmt
cargo fmt --check   # verify

# 4. Review changes
git diff
```

## Quality Standards

**Code Quality**:
- Zero clippy warnings (fix, never suppress)
- All code formatted with `cargo fmt`
- Rust 2024 edition
- `#![deny(warnings, missing_docs)]` enforced

**Test Coverage**:
- Unit tests for pure logic (in `#[cfg(test)]` modules)
- Integration tests for CLI commands (`tests/integration_test.rs`)
- Demo script tests for dogfooding (`demo/*.sh` run via `cargo test`)
- Edge case handling

**Size Limits**:
- Files under 500 lines (prefer 200-300)
- Functions under 50 lines (prefer 10-30)
- No TODO/FIXME comments left unresolved

## Dogfooding

reg-rs tests itself. The demo scripts are run as part of `cargo test`:

- `demo/dogfood.sh` — Creates regression tests for each subcommand's help output
- `demo/test_basic.sh` — Basic create/run/report/remove lifecycle
- `demo/test_workflow.sh` — Regression detection scenario

Scripts accept `REG_RS_BIN` env var to use any binary (integration tests use debug build; standalone uses release).

## Commit Messages

**Format**: `type: Short summary`

Types: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`

## Release Process

```bash
# 1. Verify all checks pass
cargo test && cargo clippy --tests && cargo fmt --check

# 2. Update version in Cargo.toml
# 3. Commit and tag
git commit -m "chore: Bump version to X.Y.Z"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push && git push --tags

# 4. Build release
cargo build --release
```
