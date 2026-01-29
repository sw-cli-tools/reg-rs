#!/bin/bash
# Test script for workflow demo - run this to verify commands work before VHS recording
set -e

cd "$(dirname "$0")/.."

echo "=== RTT1 Regression Detection Test ==="

# Clean up any previous test artifacts
rm -rf data/testdata data/version_test.tdb data/version_test.tdb.lock

# Build
echo "Building rtt1..."
cargo build --release

# Ensure data directory exists
mkdir -p data/testdata

# Create baseline test
echo ""
echo "=== Step 1: Create a baseline test ==="
echo 'version 1.0.0' > data/testdata/version.txt
./target/release/rtt1 create -t data/version_test.tdb -c 'cat data/testdata/version.txt'

# Run test - should pass
echo ""
echo "=== Step 2: Run the test - should pass (no changes) ==="
./target/release/rtt1 run -p version_test

# Report - no differences
echo ""
echo "=== Step 3: Report results - no differences ==="
./target/release/rtt1 report -p version_test -vvv

# Simulate a change (regression)
echo ""
echo "=== Step 4: Simulate a regression (version change) ==="
echo 'version 2.0.0' > data/testdata/version.txt

# Run test again - should detect regression
echo ""
echo "=== Step 5: Run the test again - detects the change ==="
./target/release/rtt1 run -p version_test

# Report with differences
echo ""
echo "=== Step 6: Report shows the differences ==="
./target/release/rtt1 report -p version_test -vvv

# Clean up
echo ""
echo "=== Cleaning up ==="
rm -rf data/testdata data/version_test.tdb data/version_test.tdb.lock

echo ""
echo "=== Done! RTT1 successfully detected the regression ==="
