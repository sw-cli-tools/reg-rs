# reg-rs Command Reference

Full reference for all `reg-rs` subcommands and flags. For a quick-start guide using shell aliases, see [README.md](../README.md).

## Table of Contents

- [create](#create)
- [run](#run)
- [list](#list)
- [show](#show)
- [report](#report)
- [rebase](#rebase)
- [remove](#remove)
- [reset](#reset)
- [migrate](#migrate)
- [status](#status)
- [analyze](#analyze)
- [Exit Codes](#exit-codes)

## create

Creates a new test by running a command and storing its output as the baseline. Writes `.rgt` format directly (TOML spec + `.out`/`.err` baselines).

```bash
reg-rs create -t <test_name> -c <command>

# Options:
#   -t, --test <name>      Test name (stored as name.rgt in data dir)
#   -c, --command <cmd>    Command to execute and capture
#   --desc <text>          Human-readable description
#   --expects <text>       Expected behavior documentation
#   --flaky-note <text>    Known flakiness notes
#   --timeout <secs>       Command timeout (default: 300)
#   -P, --preprocess <cmd> Shell command to normalize output before diffing
#   -M, --diff-mode <mode> Built-in normalization: text (default), json, lines-unordered
#   -D, --describe <text>  AI-generated command from natural language (requires ANTHROPIC_API_KEY)
#   --context <cmd>        Run command and include output in AI prompt (requires --describe)

# Examples:
reg-rs create -t pwd_test -c "pwd"
reg-rs create -t version_test -c "git --version"
reg-rs create -t api_test -c "curl -s localhost/api" -M json
reg-rs create -t log_test -c "myapp --verbose" -P "sed 's/[0-9]\{4\}/YYYY/g'"
reg-rs c -t ls_test -c "ls -la"                  # 'c' is a built-in alias
```

## run

Runs previously created tests and compares results against baselines.

```bash
reg-rs run -p <pattern> [-v|-vv] [-q] [-n]

# Options:
#   -p, --pattern <pat>    Pattern to match test names (substring match)
#   -q, --quiet            No output, exit code only (0=pass, 1=regressions)
#   -v                     Show failure details (test names + failure info)
#   -vv                    Show failure details with full diffs
#   -n, --dry-run          Print what would be run without executing
#   --parallel             Run matching tests in parallel

# Examples:
reg-rs run                                        # Run all tests (summary line)
reg-rs run -p pwd_test                            # Run a specific test
reg-rs run -p test -v                             # Run with failure details
reg-rs run -q                                     # Silent run, check exit code
reg-rs r -p pwd_test -n                           # 'r' is built-in alias; dry-run
```

## list

Lists tests with their name, command, and status (PASS/FAIL/pending).

```bash
reg-rs list                           # list all tests
reg-rs list -p my_test                # list matching tests
reg-rs l -p test                      # 'l' is built-in alias
```

## show

Shows detailed test information including command, metadata, baselines, and diffs.

```bash
reg-rs show -p my_test                # command and metadata
reg-rs show -p my_test -v             # also baseline output
reg-rs show -p my_test -vv            # also latest results and diffs
reg-rs w -p test                      # 'w' is built-in alias
```

## report

Reports on test results with configurable verbosity.

```bash
reg-rs report -p <pattern> [-v|-vv|-vvv] [-q]

# Options:
#   -p, --pattern <pat>    Pattern to match test names
#   -q, --quiet            No output, exit code only (0=pass, 1=regressions)
#   -v                     Show test names
#   -vv                    Show test names and failure info
#   -vvv                   Show test names, failures, and differences

# Examples:
reg-rs report -p pwd_test                         # Basic summary
reg-rs report -p test -v                          # Show names
reg-rs report -q                                  # Silent, check exit code
reg-rs p -p pwd_test -vvv                         # 'p' is built-in alias
```

## rebase

Accepts the latest test output as the new expected baseline.

```bash
reg-rs rebase -p my_test              # accept latest output
reg-rs u -p version                   # 'u' is built-in alias (update)

# For .rgt tests: updates .out/.err files
# For .tdb tests: replaces original results with latest
```

## remove

Removes tests and their associated files.

```bash
reg-rs remove -p <pattern>

# Examples:
reg-rs remove -p old_test
reg-rs remove -p temp_
```

## reset

Clears latest run results from the `.tdb` cache, keeping baselines intact.

```bash
reg-rs reset -p my_test               # reset matching tests
reg-rs reset                          # reset all tests
```

## migrate

Converts existing `.tdb` tests to `.rgt` text format. Only needed for legacy tests — `create` now writes `.rgt` directly.

```bash
reg-rs migrate                        # migrate all tests
reg-rs migrate -p my_test             # migrate matching tests
reg-rs m -p old_test                  # 'm' is built-in alias
```

## status

Starts a web server to monitor test results with live SSE updates.

```bash
reg-rs status -p <pattern> [-l <port>]

# Options:
#   -p, --pattern <pat>           Pattern to match tests to monitor
#   -l, --localhost-port <port>   Port number (default: 4740)

# Examples:
reg-rs status -p test
reg-rs status -p test -l 8080
reg-rs s -p test                      # 's' is built-in alias
```

Open http://localhost:4740 (or your chosen port) to view the status page.

## analyze

AI-powered failure analysis (requires `ANTHROPIC_API_KEY`).

```bash
reg-rs analyze -p failing_test
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All tests passed |
| 1 | Regressions detected |
| 2 | Error (invalid args, missing tests, etc.) |
