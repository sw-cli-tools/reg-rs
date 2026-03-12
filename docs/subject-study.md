# Subject Study: Testing CLI tools with reg-rs

This document describes how reg-rs is used to regression-test CLI tools from the [sw-cli-tools](https://github.com/sw-cli-tools) suite. It covers two case studies — pjmai-rs (text output) and favicon (binary/image output) — and serves as a reference for setting up reg-rs against your own tools.

---

# Part 1: pjmai-rs (text output)

[pjmai-rs](https://github.com/sw-cli-tools/pjmai-rs) is a Rust CLI tool for managing and switching between development projects.

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

Measured on macOS, Apple Silicon:

| Suite | Tests | Sequential | Parallel | Speedup |
|-------|-------|-----------|----------|---------|
| reg-rs self-tests | 6 | 0.098s | 0.024s | 4.1x |
| pjmai-rs | 11 | 0.186s | 0.047s | 4.0x |

Both suites achieve ~4x speedup. The parallel runs saturate multiple cores (500%+ CPU utilization) since each test spawns an independent shell process. The speedup comes from overlapping process launch and I/O wait times across tests.

With larger suites or slower commands (e.g., network-dependent tests with longer runtimes), the speedup would be more dramatic since there's more idle time to overlap.

### When to use parallel

Parallel execution is safe when:

- Each test has its own `.tdb` file (this is always the case with reg-rs)
- Test commands don't share mutable state (files, ports, databases)
- Test commands don't compete for scarce resources (e.g., all binding the same port)

The pjmai-rs suite satisfies all three: each test creates an isolated temp directory, runs pjmai-rs against it, and cleans up. No test modifies anything outside its sandbox.

### Meta-testing value

Running the pjmai-rs suite in parallel also serves as a stress test of reg-rs itself — it exercises concurrent reads and writes to different `.tdb` files, concurrent process spawning, and concurrent diff computation. Any file-locking or thread-safety bugs in reg-rs would surface as flaky failures in the parallel pjmai-rs run.

### reg-rs self-tests (dogfooding)

reg-rs also tests itself. The `demo/dogfood.sh` script creates 6 regression tests that capture the `--help` output of each reg-rs subcommand:

| Test | Command |
|------|---------|
| `reg_rs_help` | `reg-rs -h` |
| `reg_rs_create_help` | `reg-rs create -h` |
| `reg_rs_run_help` | `reg-rs run -h` |
| `reg_rs_report_help` | `reg-rs report -h` |
| `reg_rs_remove_help` | `reg-rs remove -h` |
| `reg_rs_status_help` | `reg-rs status -h` |

These tests detect regressions in CLI argument definitions — any change to flag names, help descriptions, subcommand ordering, or clap formatting will produce a diff. They are the simplest possible regression tests: no sandbox needed, no preprocessing, no state, fully deterministic.

Because they are completely independent (each just runs `reg-rs <subcommand> -h`), they run in parallel without modification and achieve the same ~4x speedup as the pjmai-rs suite. The self-tests running in parallel is a particularly good meta-test: reg-rs is simultaneously the test runner, the subject under test, and the test executor — any concurrency bug would cause reg-rs to detect a regression in itself.

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

---

# Part 2: favicon (binary/image output)

[favicon](https://github.com/sw-cli-tools/favicon) is a Rust CLI tool that generates favicon images from text, emoji, or symbols using Skia rendering. It produces PNG and ICO files — binary output that cannot be compared with line-oriented text diffs.

## Why favicon is a good subject

favicon complements pjmai-rs by exercising a completely different testing challenge: binary output verification.

- **Binary output** — PNGs are not human-readable text; line diffs are meaningless
- **Deterministic rendering** — same Skia version + same fonts + same input = byte-identical output
- **Many parameters** — symbols, colors, rotation, font selection all affect the rendered image
- **No state** — unlike pjmai-rs, favicon is stateless (no config files, no sandbox needed)
- **Visual semantics** — a human (or vision model) can look at the output and judge correctness

## Test suite overview

The suite is defined in `tests/regression/favicon_setup.sh` and creates 12 regression tests across five categories:

| Category | Tests | Strategy | Storage per test |
|----------|-------|----------|-----------------|
| Text output | 3 | Standard reg-rs text comparison | ~1 KB |
| SHA-256 checksum | 3 | Checksum for fast pass/fail | ~100 bytes |
| Base64 encoding | 2 | Full image as text (AI-decodable) | ~33 KB |
| Hybrid | 2 | Checksum + golden file on disk | ~100 bytes + ~25 KB golden |
| Error injection | 2 | Controlled failure via env var | ~100 bytes or ~33 KB |

### Text output tests

| Test | What it verifies |
|------|-----------------|
| `favicon_help` | Help text is stable |
| `favicon_version_format` | Version format (masked) |
| `favicon_list_symbols` | Symbol count is stable |

### Binary output tests

| Test | Approach | Input |
|------|----------|-------|
| `favicon_sha_heart_png` | SHA-256 | Heart emoji |
| `favicon_sha_star_rotated` | SHA-256 | Star rotated 45° |
| `favicon_sha_colored` | SHA-256 | Red A on blue background |
| `favicon_b64_heart_png` | Base64 | Heart emoji |
| `favicon_b64_rocket_png` | Base64 | Rocket emoji |
| `favicon_hybrid_heart` | Hybrid | Heart emoji |
| `favicon_hybrid_dice` | Hybrid | Dice emoji |

### Error injection tests

| Test | Approach | Purpose |
|------|----------|---------|
| `favicon_inject_heart_sha` | SHA-256 | Detects FAVICON_WATERMARK |
| `favicon_inject_heart_b64` | Base64 | Detects watermark + enables AI visual diff |

## Setup and execution

### Prerequisites

Both binaries must be built:

```bash
# In reg-rs repo
cargo build

# In favicon repo
cd ~/github/sw-cli-tools/favicon/components/app && cargo build
```

### Creating the baseline

```bash
bash tests/regression/favicon_setup.sh
```

This populates `./work/reg-rs/favicon-tests/` with one `.tdb` per test, plus a `golden/` subdirectory for hybrid tests.

### Running the tests

```bash
# Normal run (all tests should pass)
REG_RS_DATA_DIR=./work/reg-rs/favicon-tests ./target/debug/reg-rs run -p favicon --parallel

# Error injection (inject tests will fail)
FAVICON_WATERMARK=SAMPLE REG_RS_DATA_DIR=./work/reg-rs/favicon-tests ./target/debug/reg-rs run -p favicon_inject --parallel
```

### Viewing results

```bash
# Summary
REG_RS_DATA_DIR=./work/reg-rs/favicon-tests ./target/debug/reg-rs report -p favicon

# Full detail with diffs
REG_RS_DATA_DIR=./work/reg-rs/favicon-tests ./target/debug/reg-rs report -p favicon -vvv

# AI-powered failure analysis
REG_RS_DATA_DIR=./work/reg-rs/favicon-tests ./target/debug/reg-rs analyze -p favicon
```

### Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `REG_RS_BIN` | `./target/debug/reg-rs` | Path to reg-rs binary |
| `FAVICON_BIN` | `~/github/sw-cli-tools/favicon/components/app/target/debug/favicon` | Path to favicon binary |
| `REG_RS_DATA_DIR` | `./work/reg-rs/favicon-tests` | Where `.tdb` files are stored |
| `FAVICON_WATERMARK` | (unset) | When set, favicon overlays a watermark — used for error injection |

## Three approaches to binary comparison

### Approach 1: SHA-256 checksum

The simplest approach. Generate the image, compute its checksum, capture the checksum as stdout:

```bash
favicon -u heart -T --png -o $T.png 2>&1
shasum -a 256 $T.png | cut -d' ' -f1
rm -f $T.png
```

The `.tdb` stores approximately 100 bytes: the "Saved:" log line plus a 64-character hex checksum. Comparison is fast — if the checksum matches, the image is byte-identical.

**Tradeoff**: Cannot reconstruct the image from the stored data. When a test fails, you see that the checksum changed but cannot visually inspect what changed. A human (or AI) must re-run the command and manually examine the output.

### Approach 2: Base64 encoding

Encode the entire image as base64 text, which reg-rs stores and compares as regular text:

```bash
favicon -u heart -T --png -o $T.png 2>&1
base64 < $T.png
rm -f $T.png
```

The `.tdb` stores approximately 33 KB per test. Comparison is slower (string comparison of ~33K characters) but the stored data can be decoded back to an image.

**Tradeoff**: Larger storage, slower comparison. But when a test fails, both the golden and failing images can be reconstructed by decoding the base64. A vision model can then compare the two images and describe the visual difference — enabling AI-powered triage without human intervention.

### Approach 3: Hybrid (checksum + golden file)

Combine checksum comparison with a golden reference file stored on disk:

```bash
favicon -u heart -T --png -o $T.png 2>&1
cp $T.png $GOLDEN_DIR/heart.png
shasum -a 256 $T.png | cut -d' ' -f1
rm -f $T.png
```

The `.tdb` stores ~100 bytes (checksum). The golden file (~25 KB) lives in `$REG_RS_DATA_DIR/golden/`. Comparison uses the fast checksum path. When a test fails, the golden file is available for visual inspection or AI analysis.

**Tradeoff**: Requires managing files outside the `.tdb` database. The golden directory must be preserved alongside the test data. But it gets the best of both worlds: fast comparison and visual triage capability.

### Measured results

| Metric | SHA-256 | Base64 | Hybrid |
|--------|---------|--------|--------|
| `.tdb` size | ~32 KB | 98–176 KB | ~32 KB |
| Golden file | — | — | 4–25 KB |
| Comparison speed | Fast (64 chars) | Slower (~33K chars) | Fast (64 chars) |
| Image recoverable | No | Yes (decode base64) | Yes (golden file) |
| AI visual triage | No | Yes | Yes |

### Performance

| Execution | Tests | Time |
|-----------|-------|------|
| Sequential | 12 | 0.522s |
| Parallel | 12 | 0.132s |
| Speedup | — | 4.0x |

## Error injection with FAVICON_WATERMARK

favicon supports a hidden environment variable `FAVICON_WATERMARK` for controlled test failure. When set, favicon overlays a semi-transparent red diagonal watermark across the rendered image after normal rendering but before PNG encoding.

The watermark changes the image bytes → different checksum → detected regression. The watermark is visually obvious, making it ideal for testing AI visual triage: a vision model comparing the clean and watermarked images should immediately identify the red "SAMPLE" text.

### How it works in tests

Error injection tests use bash parameter expansion to conditionally pass the env var:

```bash
env ${FAVICON_WATERMARK:+FAVICON_WATERMARK=$FAVICON_WATERMARK} favicon -u heart -T --png -o $T.png 2>&1
```

When `FAVICON_WATERMARK` is unset, this expands to just `env favicon ...` (no watermark). When set to "SAMPLE", it expands to `env FAVICON_WATERMARK=SAMPLE favicon ...` (watermark applied).

This means:
- **Baseline creation**: Run without `FAVICON_WATERMARK` → clean image stored
- **Normal test run**: Run without `FAVICON_WATERMARK` → clean image → passes
- **Error injection run**: Run with `FAVICON_WATERMARK=SAMPLE` → watermarked image → fails

### Running error injection

```bash
# These two inject tests will fail (by design)
FAVICON_WATERMARK=SAMPLE REG_RS_DATA_DIR=./work/reg-rs/favicon-tests \
  ./target/debug/reg-rs run -p favicon_inject --parallel
```

The SHA test (`favicon_inject_heart_sha`) reports a checksum mismatch. The base64 test (`favicon_inject_heart_b64`) reports a base64 mismatch — and the stored base64 can be decoded for visual comparison.

## Key techniques

### Capturing stdout alongside binary verification

favicon prints a "Saved: /path/to/file.png" message to stdout. This message is valuable — it confirms the command ran successfully, and any new warnings would appear here. Rather than suppressing stdout, all tests capture it:

```bash
favicon -u heart -T --png -o $T.png 2>&1
shasum -a 256 $T.png | cut -d' ' -f1
```

The `2>&1` merges stderr into stdout so both are captured. The checksum (or base64) appears on the next line after the "Saved:" message.

### Normalizing temp paths with --preprocess

The "Saved:" message contains a PID-based temp path (`/tmp/favicon-reg-12345.png`) that changes every run. The `--preprocess` flag normalizes this:

```bash
-P "sed 's|/tmp/favicon-reg-[0-9]*|/tmp/favicon-reg-PID|g'"
```

This replaces only the variable PID portion while preserving the rest of the output. If favicon starts emitting new warnings, they will be captured and compared — producing a detected regression.

### No sandbox needed

Unlike pjmai-rs, favicon is stateless. It reads no config files, writes only to the specified output path, and has no interactive prompts. This eliminates the need for sandbox isolation — making favicon tests simpler to set up and naturally safe for parallel execution.

## Choosing an approach

| Use case | Recommended approach |
|----------|---------------------|
| CI/fast feedback | SHA-256 checksum — minimal storage, fastest comparison |
| AI-assisted triage | Base64 — image recoverable from test data alone |
| Long-lived test suites | Hybrid — fast comparison + golden files for inspection |
| Error injection demos | Both SHA + base64 — demonstrate detection and triage |

For most projects, start with SHA-256 checksums. Add base64 or hybrid tests for critical images where you want AI triage capability on failure.
