#!/bin/bash
# Dogfood script: reg-rs tests itself
# Creates regression tests for reg-rs's own CLI help output,
# then runs and reports on them.
#
# Uses work/reg-rs/ (git-ignored) to avoid polluting the user's data directory.
#
# Environment variables:
#   REG_RS_BIN       - path to reg-rs binary (default: builds release and uses ./target/release/reg-rs)
#   REG_RS_DATA_DIR  - data directory (default: ./work/reg-rs)
set -e

cd "$(dirname "$0")/.."

export REG_RS_DATA_DIR="${REG_RS_DATA_DIR:-./work/reg-rs}"
export RUST_LOG="${RUST_LOG:-warn}"

if [ -z "$REG_RS_BIN" ]; then
    echo "Building reg-rs..."
    cargo build --release 2>/dev/null
    REG_RS_BIN=./target/release/reg-rs
fi

echo "=== reg-rs Dogfood: Testing Itself ==="

# Ensure data directory exists
mkdir -p "$REG_RS_DATA_DIR"

# Clean previous self-tests
echo ""
echo "--- Cleaning previous self-tests ---"
rm -f "$REG_RS_DATA_DIR"/reg_rs_*.tdb "$REG_RS_DATA_DIR"/reg_rs_*.tdb.lock \
      "$REG_RS_DATA_DIR"/reg_rs_*.rgt "$REG_RS_DATA_DIR"/reg_rs_*.out "$REG_RS_DATA_DIR"/reg_rs_*.err

# Create self-tests for each command's help output
echo ""
echo "--- Creating self-tests ---"
"$REG_RS_BIN" create -t reg_rs_help -c "$REG_RS_BIN -h"
"$REG_RS_BIN" create -t reg_rs_create_help -c "$REG_RS_BIN create -h"
"$REG_RS_BIN" create -t reg_rs_run_help -c "$REG_RS_BIN run -h"
"$REG_RS_BIN" create -t reg_rs_report_help -c "$REG_RS_BIN report -h"
"$REG_RS_BIN" create -t reg_rs_remove_help -c "$REG_RS_BIN remove -h"
"$REG_RS_BIN" create -t reg_rs_status_help -c "$REG_RS_BIN status -h"

# Run them
echo ""
echo "--- Running self-tests ---"
"$REG_RS_BIN" run -p reg_rs

# Report results and check for failures
echo ""
echo "--- Self-test report ---"
REPORT=$("$REG_RS_BIN" report -p reg_rs -v)
echo "$REPORT"

# Extract counts from summary line
FAILED=$(echo "$REPORT" | grep -o '[0-9]* failed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//' )
PASSED=$(echo "$REPORT" | grep -o '[0-9]* passed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//' )

echo ""
if [ "${FAILED:-0}" -gt 0 ]; then
    echo "=== FAILED: ${FAILED} test(s) failed ==="
    exit 1
elif [ "${PASSED:-0}" -eq 0 ]; then
    echo "=== ERROR: No tests were run ==="
    exit 2
else
    echo "=== PASSED: All ${PASSED} self-tests passed ==="
    echo "=== Done! reg-rs successfully tested itself ==="
fi
