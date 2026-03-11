#!/bin/bash
# pjmai_setup.sh — Create regression tests for pjmai-rs
#
# Creates reg-rs regression tests that exercise pjmai-rs in an isolated
# sandbox. Each test uses its own PJMAI_CONFIG_DIR in /tmp with a
# pre-created config.toml to avoid interactive prompts.
#
# Usage:
#   bash tests/regression/pjmai_setup.sh
#
# Environment:
#   REG_RS_BIN      — path to reg-rs binary (default: ./target/debug/reg-rs)
#   PJMAI_BIN       — path to pjmai-rs binary (default: ~/github/sw-cli-tools/pjmai-rs/target/debug/pjmai-rs)
#   REG_RS_DATA_DIR — where to store .tdb files (default: ./work/reg-rs/pjmai-tests)

set -euo pipefail

REG_RS_BIN="${REG_RS_BIN:-./target/debug/reg-rs}"
PJMAI_BIN="${PJMAI_BIN:-$HOME/github/sw-cli-tools/pjmai-rs/target/debug/pjmai-rs}"
export REG_RS_DATA_DIR="${REG_RS_DATA_DIR:-./work/reg-rs/pjmai-tests}"

mkdir -p "$REG_RS_DATA_DIR"

# Verify binaries exist
if [ ! -x "$REG_RS_BIN" ]; then
  echo "ERROR: reg-rs binary not found at $REG_RS_BIN (run: cargo build)" >&2
  exit 1
fi
if [ ! -x "$PJMAI_BIN" ]; then
  echo "ERROR: pjmai-rs binary not found at $PJMAI_BIN" >&2
  exit 1
fi

echo "Creating pjmai-rs regression tests..."
echo "  reg-rs: $REG_RS_BIN"
echo "  pjmai-rs: $PJMAI_BIN"
echo "  data dir: $REG_RS_DATA_DIR"
echo ""

# Helper: create a test with optional flags
create_test() {
  local name="$1"
  local cmd="$2"
  shift 2
  echo "  creating: $name"
  "$REG_RS_BIN" create -t "$name" -c "$cmd" "$@" 2>/dev/null
}

# Sandbox preamble: create isolated config with pre-seeded config.toml
# to avoid "Create config file?" interactive prompt.
# shellcheck disable=SC2016
SANDBOX='D=$(mktemp -d) && export PJMAI_CONFIG_DIR="$D/config" && mkdir -p "$D/config" "$D/projects/alpha" "$D/projects/beta" && printf "version = \"0.1.0\"\ncurrent_project = \"\"\nstack = []\nhistory = []\nproject = []\n" > "$D/config/config.toml"'
CLEANUP='rm -rf "$D"'
P="$PJMAI_BIN"

# ============================================================
# Tests (10s timeout each — fail fast if commands hang)
# ============================================================

# TEST: Help text is stable
# EXPECTS: Usage line, subcommand list
# FLAKY: No — only changes with intentional CLI changes
# FAILURE CAUSES: CLI arg change, clap version bump
create_test "pjmai_help" \
  "$P --help 2>&1" \
  --timeout 10

# TEST: Version format (volatile parts masked)
# EXPECTS: "pjmai-rs X.Y.Z" with masked version/hash/timestamp
# FLAKY: No after masking
# FAILURE CAUSES: version format change
create_test "pjmai_version_format" \
  "$P --version 2>&1 | sed 's/[0-9]*\\.[0-9]*\\.[0-9]*/X.Y.Z/g; s/[0-9a-f]\\{7,\\}/HASH/g'" \
  --timeout 10

# TEST: Empty project list
# EXPECTS: "No projects" or empty list
# FLAKY: No — empty state is deterministic
# FAILURE CAUSES: Default output format change
create_test "pjmai_empty_list" \
  "$SANDBOX && $P list 2>&1; $CLEANUP" \
  --timeout 10

# TEST: Add project and list it
# EXPECTS: Project name appears in list output, paths stripped for stability
# FLAKY: No — deterministic with isolated config + path stripping
# FAILURE CAUSES: add/list logic change, output format change
create_test "pjmai_add_and_list" \
  "$SANDBOX && $P add -p alpha -f \"\$D/projects/alpha\" 2>&1 && $P list 2>&1; $CLEANUP" \
  --timeout 10 \
  -P "sed 's|/private/var/[^ ]*|<TMPDIR>|g; s|/var/[^ ]*|<TMPDIR>|g; s|/tmp/[^ ]*|<TMPDIR>|g'"

# TEST: Add duplicate project returns error
# EXPECTS: Error about duplicate
# FLAKY: No
# FAILURE CAUSES: Error message change, duplicate detection logic
create_test "pjmai_add_duplicate" \
  "$SANDBOX && $P add -p alpha -f \"\$D/projects/alpha\" 2>/dev/null; $P add -p alpha -f \"\$D/projects/alpha\" 2>&1; echo exit=\$?; $CLEANUP" \
  --timeout 10

# TEST: Change to nonexistent project fails
# EXPECTS: Error output about project not found
# FLAKY: No
# FAILURE CAUSES: Error message change, exit code contract
create_test "pjmai_change_nonexistent" \
  "$SANDBOX && $P change -p nonexistent 2>&1; echo exit=\$?; $CLEANUP" \
  --timeout 10

# TEST: Add and remove, verify clean
# EXPECTS: before=1, after=0
# FLAKY: No
# FAILURE CAUSES: add/remove logic
create_test "pjmai_add_remove" \
  "$SANDBOX && $P add -p alpha -f \"\$D/projects/alpha\" 2>/dev/null && echo before=\$($P list 2>/dev/null | wc -l | tr -d ' ') && $P remove -p alpha 2>/dev/null && echo after=\$($P list 2>/dev/null | wc -l | tr -d ' '); $CLEANUP" \
  --timeout 10

# TEST: Complete commands lists subcommands (sorted for stability)
# EXPECTS: Known subcommand names in sorted order
# FLAKY: No — only changes when commands added/removed
# FAILURE CAUSES: Subcommand added/removed
create_test "pjmai_complete_commands" \
  "$SANDBOX && $P complete commands 2>/dev/null | sort; $CLEANUP" \
  --timeout 10

# TEST: Change to project outputs directory path (exit code 2)
# EXPECTS: exit=2 and path is a valid directory
# FLAKY: No
# FAILURE CAUSES: Exit code contract change
create_test "pjmai_change_exit_code" \
  "$SANDBOX && $P add -p alpha -f \"\$D/projects/alpha\" 2>/dev/null; output=\$($P change -p alpha 2>/dev/null); rc=\$?; echo exit=\$rc; echo is_dir=\$(test -d \"\$output\" && echo yes || echo no); $CLEANUP" \
  --timeout 10

# TEST: Push and pop stack
# EXPECTS: after_push=beta, after_pop=alpha
# FLAKY: No
# FAILURE CAUSES: Stack logic change
create_test "pjmai_push_pop" \
  "$SANDBOX && $P add -p alpha -f \"\$D/projects/alpha\" 2>/dev/null && $P add -p beta -f \"\$D/projects/beta\" 2>/dev/null && $P change -p alpha >/dev/null 2>/dev/null; $P push -p beta >/dev/null 2>/dev/null; echo after_push=\$($P prompt 2>/dev/null); $P pop >/dev/null 2>/dev/null; echo after_pop=\$($P prompt 2>/dev/null); $CLEANUP" \
  --timeout 10

# TEST: Show command on empty project
# EXPECTS: No current project or error
# FLAKY: No
# FAILURE CAUSES: Show output format change
create_test "pjmai_show_empty" \
  "$SANDBOX && $P show 2>&1; echo exit=\$?; $CLEANUP" \
  --timeout 10

echo ""
echo "Done! Created $(ls "$REG_RS_DATA_DIR"/*.tdb 2>/dev/null | wc -l | tr -d ' ') regression tests"
echo ""
echo "Run:     REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN run -p pjmai"
echo "Report:  REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN report -p pjmai -vvv"
echo "Analyze: REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN analyze -p pjmai"
