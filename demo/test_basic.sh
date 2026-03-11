#!/bin/bash
# Test script for basic demo - run this to verify commands work before VHS recording
set -e

cd "$(dirname "$0")/.."

export REG_RS_DATA_DIR=./work/reg-rs

echo "=== reg-rs Basic Workflow Test ==="

# Build
echo "Building reg-rs..."
cargo build --release

# Ensure data directory exists
mkdir -p "$REG_RS_DATA_DIR"

# Clean up any previous test artifacts
rm -f "$REG_RS_DATA_DIR"/hello.tdb "$REG_RS_DATA_DIR"/hello.tdb.lock

# Show help
echo ""
echo "=== Step 0: Show help ==="
./target/release/reg-rs --help

# Create a test (just a name - placed in data dir automatically)
echo ""
echo "=== Step 1: Create a test ==="
./target/release/reg-rs create -t hello -c 'echo hello world'

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
