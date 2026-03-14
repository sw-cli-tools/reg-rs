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

---

# Part 3: cor24-rs (embedded emulator output)

[cor24-rs](https://github.com/sw-embed/cor24-rs) is a COR24 24-bit RISC CPU emulator with an interactive debugger (`cor24-dbg`) and a Rust-to-COR24 cross-compilation pipeline. It produces structured emulator output — register dumps, UART buffers, memory maps, and disassembly — that is fully deterministic for fixed programs.

## Why cor24-rs is a good subject

cor24-rs complements the other subjects by exercising a third testing pattern: deterministic emulator output from piped debugger commands.

- **Deterministic execution** — same program + same instruction count = identical output (register values, UART content, memory state)
- **Structured output** — emulator output has distinct sections (registers, memory, UART, I/O) that each have different stability characteristics
- **Multiple tools** — `cor24-dbg` (interactive debugger) and `cor24-run` (headless runner) produce different output formats
- **Piped stdin commands** — the debugger reads commands from stdin, enabling complex multi-step interactions (breakpoints, stepping, inspection)
- **No state or config** — emulator is stateless (no config files, no sandbox needed), similar to favicon
- **Cross-project dependency** — binaries live in a separate repo, testing the multi-project regression workflow

## Test suite overview

The suite is defined in `tests/regression/cor24_setup.sh` and creates 10 regression tests across two categories:

### Assembler demos via cor24-dbg (6 tests)

| Test | What it verifies | Key output |
|------|-----------------|------------|
| `cor24_hello_world` | UART output from assembled program | "Hello, World!" after 93 instructions |
| `cor24_count_down` | Loop counting with UART writes | "54321" after 36 instructions |
| `cor24_led_blink` | Hardware I/O peripheral toggling | "LLLLL" UART, LED D2 state |
| `cor24_debug_session` | Breakpoints, stepping, register inspection | Disassembly, breakpoint hit, r1 value |
| `cor24_disassembly` | Instruction decoder and formatter | 20 decoded instructions with mnemonics |
| `cor24_sieve` | Compute-intensive benchmark (1M instructions) | "1000 iterations" UART, stops at limit |

### Rust pipeline demos via cor24-run (4 tests)

| Test | What it verifies | Key output |
|------|-----------------|------------|
| `cor24_rust_add` | Arithmetic (42+66) in registers | r2=0x6C (108), 12 instructions |
| `cor24_rust_uart_hello` | UART writes from Rust program | UART TX log: "Hello\n" |
| `cor24_rust_fibonacci` | Recursive computation | LED=0x59, 4764 instructions |
| `cor24_rust_countdown` | Loop with memory-mapped I/O | 80332 instructions, halted |

## Setup and execution

### Prerequisites

Three binaries must be built:

```bash
# In reg-rs repo
cargo build

# In cor24-rs repo — debugger
cd ~/github/sw-embed/cor24-rs && cargo build -p cor24-cli

# In cor24-rs repo — headless runner (used by Rust pipeline demos)
cd ~/github/sw-embed/cor24-rs/rust-to-cor24 && cargo build --release
```

### Creating the baseline

```bash
bash tests/regression/cor24_setup.sh
```

This populates `./work/reg-rs/cor24-tests/` with one `.tdb` file per test.

### Running the tests

```bash
# Sequential
REG_RS_DATA_DIR=./work/reg-rs/cor24-tests ./target/debug/reg-rs run -p cor24

# Parallel (safe — no shared state)
REG_RS_DATA_DIR=./work/reg-rs/cor24-tests ./target/debug/reg-rs run -p cor24 --parallel
```

### Viewing results

```bash
# Summary
REG_RS_DATA_DIR=./work/reg-rs/cor24-tests ./target/debug/reg-rs report -p cor24

# Full detail with diffs
REG_RS_DATA_DIR=./work/reg-rs/cor24-tests ./target/debug/reg-rs report -p cor24 -vvv

# AI-powered failure analysis
REG_RS_DATA_DIR=./work/reg-rs/cor24-tests ./target/debug/reg-rs analyze -p cor24
```

### Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `REG_RS_BIN` | `./target/debug/reg-rs` | Path to reg-rs binary |
| `COR24_DIR` | `~/github/sw-embed/cor24-rs` | Path to cor24-rs repo root |
| `REG_RS_DATA_DIR` | `./work/reg-rs/cor24-tests` | Where `.tdb` files are stored |

## Key techniques

### Piped stdin for debugger interaction

The `cor24-dbg` debugger reads commands from stdin. Tests pipe multi-line command sequences via heredoc:

```bash
cor24-dbg program.lgo <<'CMDS'
run 1000
uart
quit
CMDS
```

This enables testing complex debugger workflows — breakpoints, stepping, register inspection — as a single deterministic command. The heredoc approach is critical: it makes interactive debugging sessions reproducible.

### Path masking with --preprocess

The "Loaded N bytes from /full/path/to/file.lgo" line contains an absolute path that varies by machine. The `--preprocess` flag normalizes this:

```bash
-P "sed 's|$COR24_DIR|<COR24>|g'"
```

Note: `$COR24_DIR` is expanded at test creation time (not at runtime) because the sed pattern is baked into the `.tdb` database. This means the baseline and runtime machine must use the same path, or the path masking must cover both forms. For CI, set `COR24_DIR` to a consistent path.

### Instruction counts as regression signals

The emulator is fully deterministic: the same program always executes the same number of instructions. This makes instruction counts a powerful regression signal:

- `hello_world`: exactly 93 instructions
- `count_down`: exactly 36 instructions
- `led_blink`: exactly 1577 instructions
- `rust_fibonacci`: exactly 4764 instructions

A change in instruction count means the emulator's execution path changed — either the program was modified or the emulator has a bug. Unlike timing-based metrics, instruction counts are stable across different machines and CPU loads.

### Two emulator front-ends, one test suite

The suite tests both `cor24-dbg` and `cor24-run`:

- **cor24-dbg** (5 tests): Interactive debugger. Tests pipe stdin commands. Output includes `(cor24)` prompts, structured command responses, UART buffer display.
- **cor24-run** (4 tests): Headless runner. Tests pass `--dump` flag for full state dump. Output includes register dump, memory hex dump, UART TX log, LED/I/O state.

Testing both front-ends catches regressions in the shared emulator core (instruction execution, peripherals) as well as each front-end's output formatting.

### Pre-compiled assembly files

The Rust pipeline demos use pre-compiled `.cor24.s` assembly files rather than running the full Rust→MSP430→COR24 compilation pipeline. This has three benefits:

1. **Speed** — compiling Rust with `-Z build-std=core` for MSP430 takes 10+ seconds per demo; running the pre-compiled assembly takes milliseconds
2. **Stability** — Rust nightly compiler changes would cause false positives in reg-rs tests
3. **Isolation** — tests verify the assembler and emulator, not the Rust compiler or MSP430 translator

When the Rust compiler or translator changes, regenerate the `.cor24.s` files using `run-demo.sh` in the cor24-rs repo, then re-baseline with `bash tests/regression/cor24_setup.sh`.

### No sandbox needed

Like favicon, cor24-rs is stateless. The emulator reads a binary file, executes it, and writes output to stdout. No config files, no persistent state, no interactive prompts. This makes the tests trivially safe for parallel execution.

## What the tests catch

| Change | Tests affected | What the diff shows |
|--------|---------------|-------------------|
| ALU instruction bug | `cor24_rust_add`, `cor24_rust_fibonacci` | Wrong register values, different instruction count |
| UART peripheral change | `cor24_hello_world`, `cor24_count_down`, `cor24_rust_uart_hello` | Missing or wrong UART content |
| LED I/O mapping change | `cor24_led_blink`, `cor24_rust_fibonacci` | Wrong LED state bits |
| Disassembler formatting | `cor24_disassembly`, `cor24_debug_session` | Changed mnemonic format or alignment |
| Breakpoint logic | `cor24_debug_session` | Breakpoint not hit, wrong register at break |
| Memory layout change | `cor24_rust_*` | Different memory dump, stack pointer |
| Program loading bug | All tests | Wrong byte count, different execution |

## Extending the suite

The cor24-rs repo has additional test programs and demos that could be added:

```bash
# Additional assembler programs
ls ~/github/sw-embed/cor24-rs/tests/programs/*.lgo
# hello_uart.lgo, led_on.lgo

# Additional Rust pipeline demos (13 total, 4 tested)
ls ~/github/sw-embed/cor24-rs/rust-to-cor24/demos/demo_*/
# demo_blinky, demo_button_echo, demo_drop, demo_echo,
# demo_echo_v2, demo_fibonacci_iter, demo_nested,
# demo_panic, demo_stack_vars

# Sieve benchmark (500M instructions — slower, good for timing regression)
cor24-dbg --entry 0x93 sieve.lgo <<< "run 500_000_000\nuart\nquit"
```

To add a test, append to `cor24_setup.sh` following the pattern:

```bash
create_test "cor24_new_test" \
  "$DBG $PROGRAMS/new_program.lgo <<'CMDS'
run 1000
uart
quit
CMDS" \
  --timeout 10 \
  -P "sed 's|$COR24_DIR|<COR24>|g'" \
  --desc "What this test verifies" \
  --expects "Expected output description"
```
