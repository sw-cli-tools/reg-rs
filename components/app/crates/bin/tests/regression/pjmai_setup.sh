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
export RUST_LOG="${RUST_LOG:-warn}"

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

echo "=== pjmai-rs Regression Tests ==="
echo "  reg-rs:   $REG_RS_BIN"
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
# Create tests
# ============================================================

echo "--- Creating tests ---"

create_test "pjmai_help" \
  "$P --help 2>&1" \
  --timeout 10

create_test "pjmai_version_format" \
  "$P --version 2>&1 | sed 's/[0-9]*\\.[0-9]*\\.[0-9]*/X.Y.Z/g; s/[0-9a-f]\\{7,\\}/HASH/g'" \
  --timeout 10

create_test "pjmai_empty_list" \
  "$SANDBOX && $P list 2>&1; $CLEANUP" \
  --timeout 10

create_test "pjmai_add_and_list" \
  "$SANDBOX && $P add -p alpha -f \"\$D/projects/alpha\" 2>&1 && $P list 2>&1; $CLEANUP" \
  --timeout 10 \
  -P "sed 's|/private/var/[^ ]*|<TMPDIR>|g; s|/var/[^ ]*|<TMPDIR>|g; s|/tmp/[^ ]*|<TMPDIR>|g'"

create_test "pjmai_add_duplicate" \
  "$SANDBOX && $P add -p alpha -f \"\$D/projects/alpha\" 2>/dev/null; $P add -p alpha -f \"\$D/projects/alpha\" 2>&1; echo exit=\$?; $CLEANUP" \
  --timeout 10

create_test "pjmai_change_nonexistent" \
  "$SANDBOX && $P change -p nonexistent 2>&1; echo exit=\$?; $CLEANUP" \
  --timeout 10

create_test "pjmai_add_remove" \
  "$SANDBOX && $P add -p alpha -f \"\$D/projects/alpha\" 2>/dev/null && echo before=\$($P list 2>/dev/null | wc -l | tr -d ' ') && $P remove -p alpha 2>/dev/null && echo after=\$($P list 2>/dev/null | wc -l | tr -d ' '); $CLEANUP" \
  --timeout 10

create_test "pjmai_complete_commands" \
  "$SANDBOX && $P complete commands 2>/dev/null | sort; $CLEANUP" \
  --timeout 10

create_test "pjmai_change_exit_code" \
  "$SANDBOX && $P add -p alpha -f \"\$D/projects/alpha\" 2>/dev/null; output=\$($P change -p alpha 2>/dev/null); rc=\$?; echo exit=\$rc; echo is_dir=\$(test -d \"\$output\" && echo yes || echo no); $CLEANUP" \
  --timeout 10

create_test "pjmai_push_pop" \
  "$SANDBOX && $P add -p alpha -f \"\$D/projects/alpha\" 2>/dev/null && $P add -p beta -f \"\$D/projects/beta\" 2>/dev/null && $P change -p alpha >/dev/null 2>/dev/null; $P push -p beta >/dev/null 2>/dev/null; echo after_push=\$($P prompt 2>/dev/null); $P pop >/dev/null 2>/dev/null; echo after_pop=\$($P prompt 2>/dev/null); $CLEANUP" \
  --timeout 10

create_test "pjmai_show_empty" \
  "$SANDBOX && $P show 2>&1; echo exit=\$?; $CLEANUP" \
  --timeout 10

# ============================================================
# Run tests and report
# ============================================================

echo ""
echo "--- Running tests ---"
"$REG_RS_BIN" run -p pjmai

echo ""
echo "--- Report ---"
REPORT=$("$REG_RS_BIN" report -p pjmai -v)
echo "$REPORT"

FAILED=$(echo "$REPORT" | grep -o '[0-9]* failed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')
PASSED=$(echo "$REPORT" | grep -o '[0-9]* passed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')

echo ""
if [ "${FAILED:-0}" -gt 0 ]; then
    echo "=== FAILED: ${FAILED} pjmai test(s) failed ==="
    echo ""
    echo "To see diffs:   REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN show -p pjmai -vv"
    echo "To rebase:      REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN rebase -p pjmai"
    exit 1
elif [ "${PASSED:-0}" -eq 0 ]; then
    echo "=== ERROR: No tests were run ==="
    exit 2
else
    echo "=== PASSED: All ${PASSED} pjmai test(s) passed ==="
fi
