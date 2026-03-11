#!/bin/bash
# Test script for workflow demo - run this to verify commands work before VHS recording
set -e

cd "$(dirname "$0")/.."

export REG_RS_DATA_DIR=./work/reg-rs

echo "=== reg-rs Regression Detection Test ==="

# Build
echo "Building reg-rs..."
cargo build --release

# Ensure data directory exists
mkdir -p "$REG_RS_DATA_DIR"/testdata

# Clean up any previous test artifacts
rm -f "$REG_RS_DATA_DIR"/version_test.tdb "$REG_RS_DATA_DIR"/version_test.tdb.lock
rm -rf "$REG_RS_DATA_DIR"/testdata/*

# Create baseline test
echo ""
echo "=== Step 1: Create a baseline test ==="
echo 'version 1.0.0' > "$REG_RS_DATA_DIR"/testdata/version.txt
./target/release/reg-rs create -t version_test -c "cat $REG_RS_DATA_DIR/testdata/version.txt"

# Run test - should pass
echo ""
echo "=== Step 2: Run the test - should pass (no changes) ==="
./target/release/reg-rs run -p version_test

# Report - no differences
echo ""
echo "=== Step 3: Report results - no differences ==="
./target/release/reg-rs report -p version_test -vvv

# Simulate a change (regression)
echo ""
echo "=== Step 4: Simulate a regression (version change) ==="
echo 'version 2.0.0' > "$REG_RS_DATA_DIR"/testdata/version.txt

# Run test again - should detect regression
echo ""
echo "=== Step 5: Run the test again - detects the change ==="
./target/release/reg-rs run -p version_test

# Report with differences
echo ""
echo "=== Step 6: Report shows the differences ==="
./target/release/reg-rs report -p version_test -vvv

# Clean up
echo ""
echo "=== Cleaning up ==="
rm -rf "$REG_RS_DATA_DIR"/testdata "$REG_RS_DATA_DIR"/version_test.tdb "$REG_RS_DATA_DIR"/version_test.tdb.lock

echo ""
echo "=== Done! reg-rs successfully detected the regression ==="
