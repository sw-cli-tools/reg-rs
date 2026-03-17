#!/bin/bash
# run_all.sh — Run all regression test suites and report combined results
#
# Discovers tests across all project subdirectories under work/reg-rs/
# and produces one unified summary.
#
# Usage:
#   bash tests/regression/run_all.sh           # run all tests
#   bash tests/regression/run_all.sh cor24     # run only cor24 tests
#   bash tests/regression/run_all.sh --list    # list all tests without running
#
# Environment:
#   REG_RS_BIN      — path to reg-rs binary (default: ./target/release/reg-rs)
#   REG_RS_DATA_DIR — root data directory (default: ./work/reg-rs)

set -euo pipefail

cd "$(dirname "$0")/../.."

REG_RS_BIN="${REG_RS_BIN:-./target/release/reg-rs}"
export REG_RS_DATA_DIR="${REG_RS_DATA_DIR:-./work/reg-rs}"
export RUST_LOG="${RUST_LOG:-warn}"

if [ ! -x "$REG_RS_BIN" ]; then
    echo "Building reg-rs..."
    cargo build --release 2>/dev/null
fi

PATTERN="${1:-.rgt}"

# List mode
if [ "$PATTERN" = "--list" ]; then
    "$REG_RS_BIN" list
    exit 0
fi

echo "=== reg-rs Regression Test Suite ==="
echo "  binary:   $REG_RS_BIN"
echo "  data dir: $REG_RS_DATA_DIR"
echo "  pattern:  $PATTERN"
echo ""

# Run
echo "--- Running tests ---"
"$REG_RS_BIN" run -p "$PATTERN" --parallel

# Report
echo ""
echo "--- Report ---"
REPORT=$("$REG_RS_BIN" report -p "$PATTERN" -v)
echo "$REPORT"

# Parse counts
FAILED=$(echo "$REPORT" | grep -o '[0-9]* failed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')
PASSED=$(echo "$REPORT" | grep -o '[0-9]* passed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')
PENDING=$(echo "$REPORT" | grep -o '[0-9]* not yet run' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')
TOTAL=$(echo "$REPORT" | grep -o '[0-9]* matched' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')

echo ""
echo "==============================="
echo "  Total:   ${TOTAL:-0}"
echo "  Passed:  ${PASSED:-0}"
echo "  Failed:  ${FAILED:-0}"
echo "  Pending: ${PENDING:-0}"
echo "==============================="

if [ "${FAILED:-0}" -gt 0 ]; then
    echo ""
    echo "=== FAILED: ${FAILED} test(s) failed ==="
    echo ""
    echo "To see diffs:  $REG_RS_BIN show -p '$PATTERN' -vv"
    echo "To rebase all: $REG_RS_BIN rebase -p '$PATTERN'"
    exit 1
elif [ "${PASSED:-0}" -eq 0 ]; then
    echo ""
    echo "=== ERROR: No tests were run ==="
    exit 2
else
    echo ""
    echo "=== PASSED: All ${PASSED} test(s) passed ==="
fi
