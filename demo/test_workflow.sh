#!/bin/bash
# Test script for workflow demo - run this to verify commands work before VHS recording
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

echo "=== reg-rs Regression Detection Test ==="

# Ensure data directory exists
mkdir -p "$REG_RS_DATA_DIR"/testdata

# Clean up any previous test artifacts
rm -f "$REG_RS_DATA_DIR"/version_test.tdb "$REG_RS_DATA_DIR"/version_test.tdb.lock \
      "$REG_RS_DATA_DIR"/version_test.rgt "$REG_RS_DATA_DIR"/version_test.out "$REG_RS_DATA_DIR"/version_test.err
rm -rf "$REG_RS_DATA_DIR"/testdata/*

# Create baseline test
echo ""
echo "--- Step 1: Create a baseline test ---"
echo 'version 1.0.0' > "$REG_RS_DATA_DIR"/testdata/version.txt
"$REG_RS_BIN" create -t version_test -c "cat $REG_RS_DATA_DIR/testdata/version.txt"

# Migrate to .rgt format
echo ""
echo "--- Step 2: Migrate to .rgt format ---"
"$REG_RS_BIN" migrate -p version_test

# Run test - should pass
echo ""
echo "--- Step 3: Run the test - should pass ---"
"$REG_RS_BIN" run -p version_test

# List - shows status
echo ""
echo "--- Step 4: List shows PASS ---"
"$REG_RS_BIN" list -p version_test

# Simulate a change (regression)
echo ""
echo "--- Step 5: Simulate a regression (version change) ---"
echo 'version 2.0.0' > "$REG_RS_DATA_DIR"/testdata/version.txt

# Run test again - should detect regression
echo ""
echo "--- Step 6: Run the test again - detects the change ---"
"$REG_RS_BIN" run -p version_test

# Show with diffs
echo ""
echo "--- Step 7: Show details with diffs ---"
"$REG_RS_BIN" show -p version_test -vv

# Rebase - accept the new output
echo ""
echo "--- Step 8: Rebase - accept new output as baseline ---"
"$REG_RS_BIN" rebase -p version_test

# Run again - should pass now
echo ""
echo "--- Step 9: Run again - passes with new baseline ---"
"$REG_RS_BIN" run -p version_test

# Report and check
echo ""
echo "--- Final report ---"
REPORT=$("$REG_RS_BIN" report -p version_test -v)
echo "$REPORT"

# Clean up
echo ""
echo "--- Cleaning up ---"
rm -rf "$REG_RS_DATA_DIR"/testdata \
       "$REG_RS_DATA_DIR"/version_test.tdb "$REG_RS_DATA_DIR"/version_test.tdb.lock \
       "$REG_RS_DATA_DIR"/version_test.rgt "$REG_RS_DATA_DIR"/version_test.out "$REG_RS_DATA_DIR"/version_test.err

# Check results
FAILED=$(echo "$REPORT" | grep -o '[0-9]* failed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')
PASSED=$(echo "$REPORT" | grep -o '[0-9]* passed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')

echo ""
if [ "${FAILED:-0}" -gt 0 ]; then
    echo "=== FAILED: ${FAILED} test(s) failed ==="
    exit 1
elif [ "${PASSED:-0}" -eq 0 ]; then
    echo "=== ERROR: No tests were run ==="
    exit 2
else
    echo "=== PASSED: All ${PASSED} test(s) passed ==="
    echo "=== Done! reg-rs successfully detected the regression ==="
fi
