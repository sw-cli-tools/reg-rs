#!/bin/bash
# Test script for basic demo - run this to verify commands work before VHS recording
set -e

cd "$(dirname "$0")/.."

echo "=== RTT1 Basic Workflow Test ==="

# Clean up any previous test artifacts
rm -f data/hello.tdb data/hello.tdb.lock

# Build
echo "Building rtt1..."
cargo build --release

# Ensure data directory exists
mkdir -p data

# Show help
echo ""
echo "=== Step 0: Show help ==="
./target/release/rtt1 --help

# Create a test (must be in data/ directory with .tdb extension)
echo ""
echo "=== Step 1: Create a test ==="
./target/release/rtt1 create -t data/hello.tdb -c 'echo hello world'

# Run the test
echo ""
echo "=== Step 2: Run the test ==="
./target/release/rtt1 run -p hello

# Report results
echo ""
echo "=== Step 3: Report on test results ==="
./target/release/rtt1 report -p hello -v

# Clean up
echo ""
echo "=== Step 4: Remove the test ==="
./target/release/rtt1 remove -p hello

echo ""
echo "=== Done! All steps completed successfully ==="
