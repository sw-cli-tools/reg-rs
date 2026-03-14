# Known Gaps

## ~~No way to view or edit stored test commands~~

Test commands are stored inside each `.tdb` SQLite database in the
`original_results_table`. There is currently no `reg-rs` subcommand to
inspect or modify them.

**Workaround:**

```bash
# View stored command
sqlite3 my_test.tdb "SELECT command FROM original_results_table"

# View all stored fields
sqlite3 my_test.tdb "SELECT name, command, time_created, exit_code, stdout, stderr FROM original_results_table"

# View metadata (desc, expects, flaky_note)
sqlite3 my_test.tdb "SELECT key, value FROM metadata_table"
```

To change a test command, remove and recreate:

```bash
reg-rs remove -p my_test
reg-rs create -t my_test -c "new command here"
```

**Partial fix (implemented):** The `list` subcommand shows test names, commands, and status:

```bash
reg-rs list                     # list all tests with status
reg-rs list -p my_test          # list matching tests
reg-rs l -p my_test             # using alias
```

**Resolved.** The `show` subcommand displays full test detail:

```bash
reg-rs show -p my_test          # command, metadata, status
reg-rs show -p my_test -v       # also baseline stdout/stderr
reg-rs show -p my_test -vv      # also latest results and diffs
reg-rs w -p my_test             # alias
```

## Pattern matching is substring-only

The `-p` pattern flag uses literal substring matching on the full file path,
not regex or glob. This means `-p .*` looks for the literal string `.*`, not
"match everything."

**Workaround:** Use `-p .tdb` to match all tests, or `-p ""` for everything.

**Proposed fix:** Support glob patterns or regex with a flag.

## ~~No way to run tests under the current working directory without REG_RS_DATA_DIR~~

**Resolved.** reg-rs now auto-discovers test databases. Resolution order:

1. `$REG_RS_DATA_DIR` (if set)
2. `./work/reg-rs/` (if it exists)
3. Current directory (if it contains `.tdb` files)
4. `~/.local/reg-rs/` (default)

The `-p` pattern is now optional (defaults to all tests). Simplest usage:

```bash
cd my-project
reg-rs run              # discovers ./work/reg-rs/, runs all tests
reg-rs run -p pjmai     # runs only matching tests
reg-rs report           # shows summary of all tests
```
