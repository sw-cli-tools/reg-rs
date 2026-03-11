#!/bin/bash
# Test script for workflow demo - run this to verify commands work before VHS recording
#
# Environment variables:
#   REG_RS_BIN       - path to reg-rs binary (default: builds release and uses ./target/release/reg-rs)
#   REG_RS_DATA_DIR  - data directory (default: ./work/reg-rs)
set -e

cd "$(dirname "$0")/.."

export REG_RS_DATA_DIR="${REG_RS_DATA_DIR:-./work/reg-rs}"

if [ -z "$REG_RS_BIN" ]; then
    echo "Building reg-rs..."
    cargo build --release
    REG_RS_BIN=./target/release/reg-rs
fi

echo "=== reg-rs Regression Detection Test ==="

# Ensure data directory exists
mkdir -p "$REG_RS_DATA_DIR"/testdata

# Clean up any previous test artifacts
rm -f "$REG_RS_DATA_DIR"/version_test.tdb "$REG_RS_DATA_DIR"/version_test.tdb.lock
rm -rf "$REG_RS_DATA_DIR"/testdata/*

# Create baseline test
echo ""
echo "=== Step 1: Create a baseline test ==="
echo 'version 1.0.0' > "$REG_RS_DATA_DIR"/testdata/version.txt
"$REG_RS_BIN" create -t version_test -c "cat $REG_RS_DATA_DIR/testdata/version.txt"

# Run test - should pass
echo ""
echo "=== Step 2: Run the test - should pass (no changes) ==="
"$REG_RS_BIN" run -p version_test

# Report - no differences
echo ""
echo "=== Step 3: Report results - no differences ==="
"$REG_RS_BIN" report -p version_test -vvv

# Simulate a change (regression)
echo ""
echo "=== Step 4: Simulate a regression (version change) ==="
echo 'version 2.0.0' > "$REG_RS_DATA_DIR"/testdata/version.txt

# Run test again - should detect regression
echo ""
echo "=== Step 5: Run the test again - detects the change ==="
"$REG_RS_BIN" run -p version_test

# Report with differences
echo ""
echo "=== Step 6: Report shows the differences ==="
"$REG_RS_BIN" report -p version_test -vvv

# Clean up
echo ""
echo "=== Cleaning up ==="
rm -rf "$REG_RS_DATA_DIR"/testdata "$REG_RS_DATA_DIR"/version_test.tdb "$REG_RS_DATA_DIR"/version_test.tdb.lock

echo ""
echo "=== Done! reg-rs successfully detected the regression ==="
