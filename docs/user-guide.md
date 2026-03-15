# reg-rs User Guide

## What is reg-rs?

reg-rs (pronounced "regress") captures command output as a baseline, then compares future runs against it to detect regressions. If the output changes, reg-rs tells you what changed.

## Getting Started

### Install

```bash
git clone https://github.com/sw-cli-tools/reg-rs.git
cd reg-rs
cargo install --path .
```

### Create your first test

```bash
reg-rs create -t hello -c "echo hello world"
```

This runs `echo hello world`, captures stdout/stderr/exit code, and stores the results as the baseline.

### Run the test

```bash
reg-rs run -p hello
```

Compares the current output against the baseline. If `echo hello world` still outputs `hello world`, it passes. By default, `run` prints a one-line summary; use `-v` for failure details or `-q` for silent mode (exit code only: 0=pass, 1=regressions).

### Check results

```bash
reg-rs list              # quick overview: PASS/FAIL/pending
reg-rs show -p hello     # detailed view
reg-rs report -v         # formal report with names
```

## Test Formats

### .rgt format (recommended)

Tests are defined as TOML `.rgt` files with companion `.out`/`.err` baselines. These are plain text files that work well with git:

```
my_test.rgt     # test spec (command, settings, metadata)
my_test.out     # expected stdout
my_test.err     # expected stderr (absent if empty)
my_test.tdb     # runtime cache (gitignored)
```

Example `.rgt` file:

```toml
command = "myapp --version"
exit_code = 0
desc = "Version string format check"
expects = "Prints version in semver format"
```

#### Optional .rgt fields

| Field | Default | Purpose |
|-------|---------|---------|
| `command` | (required) | Shell command to execute |
| `timeout` | 300 | Timeout in seconds |
| `exit_code` | (not compared) | Expected exit code; if present, mismatches are failures |
| `preprocess` | (none) | Shell command to normalize output before diffing |
| `diff_mode` | `text` | Built-in normalization: `text`, `json`, `lines-unordered` |
| `desc` | (none) | Human-readable description |
| `expects` | (none) | Expected behavior |
| `flaky_note` | (none) | Known flakiness notes |

### .tdb format (legacy)

Tests stored as SQLite databases. Everything (command, baselines, results, diffs) is in one binary file. Use `reg-rs migrate` to convert to `.rgt` format.

### Migrating from .tdb to .rgt

```bash
reg-rs migrate                    # migrate all tests
reg-rs migrate -p my_test         # migrate specific tests
```

After migrating, add to `.gitignore`:

```
*.tdb
*.lock
```

## Commands Reference

### Creating tests

```bash
# Basic: specify command directly
reg-rs create -t my_test -c "echo hello"

# With metadata
reg-rs create -t my_test -c "echo hello" \
  --desc "Tests echo output" \
  --expects "Prints hello" \
  --timeout 10

# With preprocessing (normalize output before diffing)
reg-rs create -t api_test -c "curl -s localhost:8080/api" \
  -P "jq --sort-keys"

# With diff mode (built-in normalization)
reg-rs create -t json_test -c "myapp --json" -M json

# AI-generated command (requires ANTHROPIC_API_KEY)
reg-rs create -t ls_test -D "list files in current directory sorted by size"
```

### Running tests

```bash
reg-rs run                        # run all tests (summary line)
reg-rs run -p my_test             # run matching tests
reg-rs run -v                     # show failure details
reg-rs run -vv                    # show failure details with full diffs
reg-rs run -q                     # quiet mode: no output, exit code only
reg-rs run -p test --parallel     # run all matching in parallel
reg-rs run -p my_test -n          # dry run (show what would run)
```

### Viewing results

```bash
# Quick status overview
reg-rs list                       # all tests
reg-rs list -p pjmai              # matching tests

# Detailed view
reg-rs show -p my_test            # command, metadata, status
reg-rs show -p my_test -v         # + baseline stdout/stderr
reg-rs show -p my_test -vv        # + latest results and diffs

# Formal report
reg-rs report                     # summary counts
reg-rs report -v                  # + test names
reg-rs report -vv                 # + failure info
reg-rs report -vvv                # + detailed diffs
reg-rs report -q                  # quiet mode: no output, exit code only
```

### Managing baselines

```bash
# Accept latest output as new baseline (when a change is intentional)
reg-rs rebase -p my_test

# Clear latest results (mark as pending, keep baselines)
reg-rs reset -p my_test

# Remove test entirely
reg-rs remove -p my_test
```

### Web dashboard

```bash
reg-rs status                     # start on default port 4740
reg-rs status -p pjmai -l 8080   # custom port, filtered tests
```

Open http://localhost:4740 to view live results with auto-updating via SSE.

### AI analysis

```bash
# Analyze a failure (requires ANTHROPIC_API_KEY)
reg-rs analyze -p failing_test
```

## Shell Aliases

Add to your `.bashrc` or `.zshrc`:

```bash
source /path/to/reg-rs/bin/source-rg.sh
```

This gives you short commands with tab-completion:

| Alias | Action | Example |
|-------|--------|---------|
| `rnrg` | Run tests | `rnrg pjmai` |
| `adrg` | Add/create test | `adrg my_test 'echo hi'` |
| `lsrg` | List tests | `lsrg` |
| `shrg` | Show test details | `shrg my_test -v` |
| `uprg` | Rebase baseline | `uprg my_test` |
| `rsrg` | Reset results | `rsrg my_test` |
| `rmrg` | Remove test | `rmrg old_test` |
| `mgrg` | Migrate to .rgt | `mgrg` |
| `strg` | Status server | `strg` |
| `hlrg` | Show alias help | `hlrg` |

## Common Workflows

### Setting up regression tests for a project

```bash
cd my-project
mkdir -p work/reg-rs

# Create tests (reg-rs auto-discovers work/reg-rs/)
# create writes .rgt format directly — no migrate step needed
reg-rs create -t version -c "myapp --version"
reg-rs create -t help -c "myapp --help"
reg-rs create -t basic -c "myapp process input.txt"

# Run all tests
reg-rs run

# Track in git
echo "*.tdb" >> .gitignore
echo "*.lock" >> .gitignore
git add work/reg-rs/*.rgt work/reg-rs/*.out work/reg-rs/*.err .gitignore
git commit -m "add regression tests"
```

### Handling a regression

```bash
# Run tests — one fails
rnrg
# version: FAIL

# See what changed
shrg version -vv
# Shows: expected "1.0.0", got "1.1.0"

# If the change is intentional, accept the new baseline
uprg version
git add work/reg-rs/version.out
git commit -m "update version baseline for 1.1.0"

# If the change is a bug, fix the code and re-run
rnrg version
# version: PASS
```

### Fresh clone / CI

```bash
git clone my-project && cd my-project
reg-rs run
# .tdb cache files are created automatically
# .rgt specs and .out/.err baselines are read from git
```

### Preprocessing noisy output

```bash
# Strip timestamps before comparing
reg-rs create -t logs -c "myapp --verbose" \
  -P "sed 's/[0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}/DATE/g'"

# Sort JSON keys for stable comparison
reg-rs create -t api -c "curl -s localhost/api" -M json

# Sort lines for order-independent comparison
reg-rs create -t ls_test -c "ls /tmp" -M lines-unordered
```

### Testing with piped input

```bash
# Commands with stdin via heredoc or echo pipe
reg-rs create -t calc -c "echo '2+3' | bc"
reg-rs create -t grep_test -c "echo -e 'foo\nbar\nbaz' | grep ba"
```

## Data Directory Discovery

reg-rs finds tests automatically (in order):

1. `$REG_RS_DATA_DIR` environment variable
2. `./work/reg-rs/` (if it exists under cwd)
3. Current directory (if it contains `.tdb` or `.rgt` files)
4. `~/.local/reg-rs/` (default)

Override with: `REG_RS_DATA_DIR=/path/to/tests reg-rs run`

## Tips

- Use `--desc` and `--expects` on `create` to document what each test verifies — these show up in failure reports
- Use `--preprocess` to strip non-deterministic output (timestamps, PIDs, paths) before diffing
- Use `--parallel` for faster test runs when tests are independent
- The `-p` pattern is optional on all commands — omit it to operate on all tests
- Pattern matching is substring-based: `-p foo` matches `foobar`, `my_foo_test`, etc.
- Use `-q` on `run` or `report` for CI scripts — no output, just exit codes (0=pass, 1=regressions, 2=error)
- Debug output (`-d` flag) shows SQL queries and internal state; default log level is `warn` for clean output
