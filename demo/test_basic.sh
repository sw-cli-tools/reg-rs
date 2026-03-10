#!/bin/bash
# Test script for basic demo - run this to verify commands work before VHS recording
set -e

cd "$(dirname "$0")/.."

echo "=== reg-rs Basic Workflow Test ==="

# Clean up any previous test artifacts
rm -f data/hello.tdb data/hello.tdb.lock

# Build
echo "Building reg-rs..."
cargo build --release

# Ensure data directory exists
mkdir -p data

# Show help
echo ""
echo "=== Step 0: Show help ==="
./target/release/reg-rs --help

# Create a test (must be in data/ directory with .tdb extension)
echo ""
echo "=== Step 1: Create a test ==="
./target/release/reg-rs create -t data/hello.tdb -c 'echo hello world'

# Run the test
echo ""
echo "=== Step 2: Run the test ==="
./target/release/reg-rs run -p hello

# Report results
echo ""
echo "=== Step 3: Report on test results ==="
./target/release/reg-rs report -p hello -v

# Clean up
echo ""
echo "=== Step 4: Remove the test ==="
./target/release/reg-rs remove -p hello

echo ""
echo "=== Done! All steps completed successfully ==="
