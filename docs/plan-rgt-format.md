# Plan: .rgt Text Format and Shell Aliases

## Status: Complete (all steps implemented)

## Problem

1. **Binary .tdb files can't be tracked in git** — test definitions (command, metadata) and test results (baselines, diffs) are locked inside SQLite databases. PRs can't show test changes. Fresh clones need setup scripts to recreate tests.

2. **CLI is verbose** — every command requires `reg-rs <subcommand> -p <pattern>`, often with `REG_RS_DATA_DIR=...`. No shell completion for test names.

3. **Subcommand names conflict with test names** — `regress list` is ambiguous: is "list" a subcommand or a test pattern?

## Design

### File Format

Four file types with clear responsibilities:

| File | Purpose | Git tracked | Created by |
|------|---------|-------------|------------|
| `.rgt` | Test specification (TOML) | Yes | `adrg` or hand-edited |
| `.out` | Expected stdout (raw content) | Yes | `adrg` (first run) or `uprg` (rebase) |
| `.err` | Expected stderr (raw content, absent if empty) | Yes | `adrg` or `uprg` |
| `.tdb` | Runtime cache (latest results, diffs, timestamps) | No (.gitignore) | `rnrg` (test runner) |

### .rgt Format

TOML with these fields:

```toml
# Required
command = "echo hello"

# Optional
timeout = 10                           # seconds (default: 300)
preprocess = "sed 's/[0-9]*/N/g'"     # shell command applied to stdout/stderr before diffing
diff_mode = "text"                     # text (default), json, lines-unordered
exit_code = 0                          # if present, exit code is compared; if absent, not compared
desc = "Hello world output"            # human-readable description
expects = "Prints 'hello' to stdout"   # expected behavior
flaky_note = "None"                    # known flakiness
```

### .out and .err Files

Raw content, exactly as captured (after preprocessing if specified). No encoding, no escaping, no delimiters. The file boundary IS the delimiter.

- `.out` always exists (even if empty — an empty stdout is a valid expectation)
- `.err` exists only if stderr baseline is non-empty (absence = expect empty stderr)

### .tdb as Cache

The `.tdb` SQLite file becomes a gitignored runtime cache:

- Stores latest run results (stdout, stderr, exit code, timestamp)
- Stores computed diffs for the web dashboard
- Rebuilt automatically when missing (fresh clone, CI)
- Web status server reads from `.tdb` for live updates

### Exit Code Handling

The `exit_code` field in `.rgt` is optional:

- **Present**: runner compares actual exit code against expected. Mismatch = failure.
- **Absent**: exit code is stored in `.tdb` for display (`shrg`) but not compared.
- **Explicit capture**: for tests that need exit code as diffable output, use `; echo exit=$?` in the command. This puts it in `.out` naturally.

### .gitignore Convention

```
*.tdb
*.lock
```

### What Goes in Git

```
work/reg-rs/hello_test.rgt
work/reg-rs/hello_test.out
work/reg-rs/hello_test.err    # only if stderr is non-empty
```

## Shell Aliases

Short commands using `*rg` suffix (avoids ripgrep `rg` conflict):

| Alias | Full form | Action |
|-------|-----------|--------|
| `rnrg [pattern]` | run regress | Run tests matching pattern (default: all) |
| `adrg <name> '<cmd>'` | add regress | Create test: write `.rgt`, run command, capture `.out`/`.err` |
| `lsrg [pattern]` | list regress | List test names, commands, status |
| `shrg <name>` | show regress | Show test spec, baseline, last result |
| `strg [pattern]` | status regress | Start web dashboard |
| `rsrg <pattern>` | reset regress | Clear latest results from `.tdb` (keep baseline) |
| `uprg <pattern>` | update regress | Accept latest output as new baseline (update `.out`/`.err`) |
| `rmrg <pattern>` | remove regress | Delete test files (`.rgt`, `.out`, `.err`, `.tdb`) |
| `hlrg` | help regress | Show alias reference |

### Alias Implementation

Shell functions in `source-rg.sh` (sourced from `.bashrc`/`.zshrc`):

```bash
rnrg() { reg-rs run -p "${1:-.rgt}" "${@:2}"; }
adrg() { reg-rs create -t "$1" -c "$2" "${@:3}"; }
lsrg() { reg-rs list -p "${1:-.rgt}"; }
shrg() { reg-rs show -p "$1"; }
strg() { reg-rs status -p "${1:-.rgt}" "${@:2}"; }
rsrg() { reg-rs reset -p "$1"; }
uprg() { reg-rs rebase -p "$1"; }
rmrg() { reg-rs remove -p "$1"; }
hlrg() { reg-rs aliases; }
```

### Shell Completion

Test name completion via `reg-rs complete tests [prefix]`:

```bash
# zsh
_rg_complete() {
  compadd $(reg-rs complete tests "$words[CURRENT]" 2>/dev/null)
}
compdef _rg_complete rnrg shrg rsrg uprg rmrg
```

Discovers test names from `.rgt` files in the data directory, including nested subdirectories.

## Workflow Examples

### Adding a test

```bash
adrg hello 'echo hello world'
# Creates: hello.rgt, hello.out
# hello.rgt contains: command = "echo hello world"
# hello.out contains: hello world

git add hello.rgt hello.out
git commit -m "add hello regression test"
```

### Running tests

```bash
rnrg              # run all tests
rnrg hello        # run tests matching "hello"
rnrg --parallel   # run all in parallel
```

### Handling a regression

```bash
rnrg
#   hello: PASS
#   version: FAIL (stdout differs)

shrg version
#   command:   myapp --version
#   exit_code: 0 (expected: 0) ✓
#   stdout:    DIFFERS
#     - myapp 1.0.0
#     + myapp 1.1.0

# If the change is intentional, accept the new baseline:
uprg version
# Updates version.out with new content

git add version.out
git commit -m "update version test baseline for 1.1.0"
```

### Fresh clone / CI

```bash
git clone myproject
cd myproject
rnrg
# .tdb files don't exist yet — runner creates them automatically
# Reads .rgt for test specs, .out/.err for expected baselines
# Runs commands, compares, stores results in .tdb cache
```

## Migration Path

### From .tdb-only to .rgt format

Add a `reg-rs migrate` command:

1. For each `.tdb` file:
   - Read command, timeout, preprocess, diff_mode, exit_code, metadata from SQLite
   - Write `.rgt` (TOML)
   - Read original stdout → write `.out`
   - Read original stderr → write `.err` (if non-empty)
2. Add `.tdb` and `.lock` to `.gitignore`
3. Setup scripts (`pjmai_setup.sh`, etc.) become optional — `.rgt` files replace them

### Coexistence

During transition, support both:
- If `.rgt` exists for a test, use it as source of truth
- If only `.tdb` exists (no `.rgt`), use legacy behavior
- `reg-rs migrate` converts `.tdb` → `.rgt`+`.out`+`.err`

## New Subcommands Required

| Subcommand | Purpose |
|------------|---------|
| `list` | Enumerate tests with status summary |
| `show` | Display test spec, baseline, and last result |
| `rebase` | Accept latest output as new baseline (update `.out`/`.err`) |
| `reset` | Clear latest results from `.tdb` cache |
| `migrate` | Convert `.tdb` files to `.rgt`+`.out`+`.err` |
| `aliases` | Print alias reference card |
| `complete` | Output test names for shell completion |

## Implementation Order

1. **`list` and `show` subcommands** — ✅ implemented
2. **`.rgt` read support** — ✅ runner reads test specs from `.rgt` files
3. **`.out`/`.err` baseline files** — ✅ runner writes and compares against text files
4. **`rebase` subcommand** — ✅ accept new baseline (update `.out`/`.err`)
5. **`migrate` subcommand** — ✅ convert existing `.tdb` to new format
6. **Shell aliases** — ✅ `bin/source-rg.sh` with functions and completions
7. **`reset` subcommand** — ✅ clear `.tdb` cache
8. **Shell completion** — ✅ `complete` subcommand + zsh/bash integration
