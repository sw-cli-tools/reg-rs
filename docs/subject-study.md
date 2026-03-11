# Subject Study: Testing pjmai-rs with reg-rs

This document describes how reg-rs is used to regression-test [pjmai-rs](https://github.com/sw-cli-tools/pjmai-rs), a Rust CLI tool for managing and switching between development projects. It serves as both a reference for maintaining the pjmai-rs test suite and a worked example for anyone setting up reg-rs against their own CLI tool.

## Why pjmai-rs is a good subject

pjmai-rs is a stateful CLI that manages a TOML config file, supports interactive prompts, communicates via exit codes (not just stdout), and has shell integration. These properties create real testing challenges:

- **Statefulness** — commands depend on prior state (add a project before you can change to it)
- **Interactive prompts** — `pjmai-rs` asks "Create config file?" when config is missing, hanging indefinitely in non-interactive shells
- **Non-deterministic output** — temp directory paths appear in output, and version strings contain build timestamps
- **Exit code contracts** — `change` returns exit code 2 (not 0) on success, printing the target directory to stdout for shell integration to `cd` into
- **Side effects** — changes persist in `~/.pjmai/config.toml`, so tests must be isolated

## Test suite overview

The suite is defined in `tests/regression/pjmai_setup.sh` and creates 11 regression tests:

| Test | What it verifies | reg-rs features used |
|------|-----------------|---------------------|
| `pjmai_help` | Help text is stable | `--timeout 10` |
| `pjmai_version_format` | Version output format (volatile parts masked via sed) | `--timeout 10` |
| `pjmai_empty_list` | Empty project list output | `--timeout 10`, sandbox |
| `pjmai_add_and_list` | Add a project, then list it | `--timeout 10`, sandbox, `--preprocess` |
| `pjmai_add_duplicate` | Duplicate add returns error | `--timeout 10`, sandbox |
| `pjmai_change_nonexistent` | Change to missing project fails | `--timeout 10`, sandbox |
| `pjmai_add_remove` | Add then remove, verify count | `--timeout 10`, sandbox |
| `pjmai_complete_commands` | Shell completions list subcommands | `--timeout 10`, sandbox |
| `pjmai_change_exit_code` | Change returns exit code 2 + valid dir | `--timeout 10`, sandbox |
| `pjmai_push_pop` | Push/pop stack navigation | `--timeout 10`, sandbox |
| `pjmai_show_empty` | Show with no active project | `--timeout 10`, sandbox |

## Setup and execution

### Prerequisites

Both binaries must be built:

```bash
# In reg-rs repo
cargo build

# In pjmai-rs repo
cd ~/github/sw-cli-tools/pjmai-rs && cargo build
```

### Creating the baseline

```bash
bash tests/regression/pjmai_setup.sh
```

This populates `./work/reg-rs/pjmai-tests/` with one `.tdb` file per test. Each `.tdb` is a SQLite database containing the captured stdout, stderr, exit code, and metadata.

### Running the tests

```bash
# Sequential (default)
REG_RS_DATA_DIR=./work/reg-rs/pjmai-tests ./target/debug/reg-rs run -p pjmai

# Parallel (one thread per test)
REG_RS_DATA_DIR=./work/reg-rs/pjmai-tests ./target/debug/reg-rs run -p pjmai --parallel
```

### Viewing results

```bash
# Summary only
REG_RS_DATA_DIR=./work/reg-rs/pjmai-tests ./target/debug/reg-rs report -p pjmai

# Full detail with diffs
REG_RS_DATA_DIR=./work/reg-rs/pjmai-tests ./target/debug/reg-rs report -p pjmai -vvv

# AI-powered failure analysis
REG_RS_DATA_DIR=./work/reg-rs/pjmai-tests ./target/debug/reg-rs analyze -p pjmai
```

### Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `REG_RS_BIN` | `./target/debug/reg-rs` | Path to reg-rs binary |
| `PJMAI_BIN` | `~/github/sw-cli-tools/pjmai-rs/target/debug/pjmai-rs` | Path to pjmai-rs binary |
| `REG_RS_DATA_DIR` | `./work/reg-rs/pjmai-tests` | Where `.tdb` files are stored |

## Key techniques

### Sandbox isolation

pjmai-rs stores state in `~/.pjmai/config.toml` by default. Tests must not touch the real config. The sandbox pattern creates an isolated environment per test invocation:

```bash
SANDBOX='D=$(mktemp -d) \
  && export PJMAI_CONFIG_DIR="$D/config" \
  && mkdir -p "$D/config" "$D/projects/alpha" "$D/projects/beta" \
  && printf "version = \"0.1.0\"\ncurrent_project = \"\"\nstack = []\nhistory = []\nproject = []\n" \
     > "$D/config/config.toml"'
CLEANUP='rm -rf "$D"'
```

Each test command is structured as: `$SANDBOX && <test commands>; $CLEANUP`

The critical detail is **pre-creating `config.toml`**. Without it, pjmai-rs prompts "Create config file? [y/n]" and waits for stdin, causing the command to hang until timeout. This was discovered when the initial test suite timed out at 300 seconds — the `--timeout 10` flag (added specifically because of this issue) makes such hangs fail in 10 seconds instead.

### Masking non-deterministic output

**Version strings** contain build-time values that change on every compile. The `pjmai_version_format` test pipes through sed to replace version numbers, git hashes, and timestamps with stable placeholders:

```bash
pjmai-rs --version 2>&1 | sed 's/[0-9]*\.[0-9]*\.[0-9]*/X.Y.Z/g; s/[0-9a-f]\{7,\}/HASH/g'
```

**Temp directory paths** appear in `list` output when projects are added with paths under `mktemp -d`. The `pjmai_add_and_list` test uses reg-rs's `--preprocess` flag to normalize paths:

```bash
-P "sed 's|/private/var/[^ ]*|<TMPDIR>|g; s|/var/[^ ]*|<TMPDIR>|g; s|/tmp/[^ ]*|<TMPDIR>|g'"
```

The `/private/var/` pattern exists because macOS resolves `/tmp` symlinks to `/private/var/folders/...` in some contexts but not others.

### Capturing exit codes as data

Several pjmai-rs commands use non-zero exit codes as part of their contract (e.g., `change` returns 2 on success for shell integration). Tests capture exit codes as stdout to make them diffable:

```bash
$P change -p nonexistent 2>&1; echo exit=$?
```

This turns the exit code into a line of text that reg-rs can compare across runs.

### Sorting for stability

The `pjmai_complete_commands` test pipes through `sort` to make output order deterministic:

```bash
$P complete commands 2>/dev/null | sort
```

For cases where the entire output has non-deterministic line order, `--diff-mode lines-unordered` is available as a built-in alternative to piping through sort.

### Counting instead of matching

The `pjmai_add_remove` test avoids comparing actual list output (which contains paths) by counting lines:

```bash
echo before=$($P list 2>/dev/null | wc -l | tr -d ' ')
$P remove -p alpha 2>/dev/null
echo after=$($P list 2>/dev/null | wc -l | tr -d ' ')
```

Output: `before=1` / `after=0` — fully deterministic, no path masking needed.

## Problems encountered and solutions

### Problem: 300-second timeout on interactive prompt

**Symptom**: Test creation hung for 5 minutes, then failed with timeout.

**Root cause**: pjmai-rs prompts "Create config file?" when `PJMAI_CONFIG_DIR` points to a directory without `config.toml`. In a non-interactive shell (no tty), stdin never provides input.

**Solution**: Pre-create a minimal `config.toml` in the sandbox. This required understanding pjmai-rs's config schema (the `project = []` array is required).

**Prevention**: The `--timeout 10` flag on every test ensures any future interactive prompt hangs are caught in 10 seconds, not 300.

### Problem: Temp directory paths differ between runs

**Symptom**: `pjmai_add_and_list` failed on every run after baseline creation. The diff showed `/tmp/tmp.qs7dNvgRdj` vs `/tmp/tmp.CW9K637OBv`.

**Root cause**: `mktemp -d` generates random directory names. The `list` command shows the full project path.

**Solution**: `--preprocess` with a sed pattern that replaces all tmpdir-like paths with `<TMPDIR>`. Three patterns are needed to cover macOS path resolution variants.

### Problem: macOS /private/var symlink resolution

**Symptom**: The initial path-stripping sed pattern (`s|/tmp/[^ ]*|...|g`) missed some paths.

**Root cause**: macOS resolves `/tmp` to `/private/var/folders/...` in some contexts. Depending on how the path was originally created vs how it's displayed, either form may appear.

**Solution**: Match all three prefixes: `/private/var/`, `/var/`, and `/tmp/`.

## Test design principles

These principles emerged from building this suite and apply to testing any CLI with reg-rs:

1. **Isolate state** — Never touch real config files. Use environment variables to redirect state to temp directories. Clean up after every test.

2. **Pre-seed required state** — If the tool has first-run prompts or setup wizards, create the expected files beforehand. Interactive prompts are the most common cause of hanging tests.

3. **Use short timeouts** — Default 300s is too long for fast CLI commands. Set `--timeout 10` (or similar) so hangs fail fast. The timeout feature exists because of this exact problem.

4. **Mask volatile output** — Timestamps, paths, versions, hashes, and PIDs all change between runs. Mask them either in the command itself (via sed/awk) or via `--preprocess`.

5. **Test contracts, not cosmetics** — Prefer testing exit codes, line counts, and key values over exact output formatting. Output format changes are usually intentional; broken logic is the real regression.

6. **Document each test** — The setup script includes comments for every test: what it expects, whether it's flaky, and what could cause failure. reg-rs also supports `--desc`, `--expects`, and `--flaky-note` flags to store this alongside the test data.

7. **Keep tests independent** — Each test creates its own sandbox. No test depends on another test's state. Tests can be run in any order or individually.

## Parallel execution

Because every test has its own `.tdb` database and its own sandbox (isolated `PJMAI_CONFIG_DIR` in a temp directory), there is no shared state between tests. This makes the pjmai-rs suite a natural fit for parallel execution.

### How it works

The `--parallel` flag on `reg-rs run` uses `std::thread::scope` to spawn one thread per test. Each thread independently:

1. Reads the original baseline from its `.tdb` file
2. Executes the test command (with timeout)
3. Computes diffs between the new output and the baseline
4. Writes results back to its `.tdb` file

Since each `.tdb` is a separate SQLite database with its own file lock, there are no concurrency conflicts. The threads share no mutable state — only the final error collection uses a mutex.

### Performance results

Measured on the 11-test pjmai-rs suite (macOS, Apple Silicon):

| Mode | Wall clock | CPU utilization |
|------|-----------|-----------------|
| Sequential | 0.179s | 90% |
| Parallel | 0.046s | 516% |

**3.9x speedup** — the parallel run saturates multiple cores since each test spawns an independent shell process. The speedup comes from overlapping the process launch and I/O wait times across tests.

With larger suites or slower commands (e.g., network-dependent tests with longer runtimes), the speedup would be more dramatic since there's more idle time to overlap.

### When to use parallel

Parallel execution is safe when:

- Each test has its own `.tdb` file (this is always the case with reg-rs)
- Test commands don't share mutable state (files, ports, databases)
- Test commands don't compete for scarce resources (e.g., all binding the same port)

The pjmai-rs suite satisfies all three: each test creates an isolated temp directory, runs pjmai-rs against it, and cleans up. No test modifies anything outside its sandbox.

### Meta-testing value

Running the pjmai-rs suite in parallel also serves as a stress test of reg-rs itself — it exercises concurrent reads and writes to different `.tdb` files, concurrent process spawning, and concurrent diff computation. Any file-locking or thread-safety bugs in reg-rs would surface as flaky failures in the parallel pjmai-rs run.

## Extending the suite

To add a new test, append to `pjmai_setup.sh`:

```bash
# TEST: Description of what this tests
# EXPECTS: What the output should look like
# FLAKY: No (or describe conditions)
# FAILURE CAUSES: What code changes would trigger this
create_test "pjmai_new_test" \
  "$SANDBOX && $P <commands> 2>&1; $CLEANUP" \
  --timeout 10
```

Then re-run the setup script to create the baseline. The old `.tdb` files are overwritten, not accumulated.

To use the new self-documenting metadata flags:

```bash
create_test "pjmai_new_test" \
  "$SANDBOX && $P <commands> 2>&1; $CLEANUP" \
  --timeout 10 \
  --desc "Verifies the frobnicate subcommand" \
  --expects "Outputs 'frobnicated' to stdout" \
  --flaky-note "None - deterministic"
```

These metadata fields are stored in the `.tdb` database and displayed in failure reports at `-vv` verbosity, helping developers (or AI agents analyzing failures via `reg-rs analyze`) understand what went wrong without reading the setup script.
