#!/bin/bash
# Test script for basic demo - run this to verify commands work before VHS recording
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

echo "=== reg-rs Basic Workflow Test ==="

# Ensure data directory exists
mkdir -p "$REG_RS_DATA_DIR"

# Clean up any previous test artifacts
rm -f "$REG_RS_DATA_DIR"/hello.tdb "$REG_RS_DATA_DIR"/hello.tdb.lock

# Show help
echo ""
echo "=== Step 0: Show help ==="
"$REG_RS_BIN" --help

# Create a test (just a name - placed in data dir automatically)
echo ""
echo "=== Step 1: Create a test ==="
"$REG_RS_BIN" create -t hello -c 'echo hello world'

# Run the test
echo ""
echo "=== Step 2: Run the test ==="
"$REG_RS_BIN" run -p hello

# Report results
echo ""
echo "=== Step 3: Report on test results ==="
"$REG_RS_BIN" report -p hello -v

# Clean up
echo ""
echo "=== Step 4: Remove the test ==="
"$REG_RS_BIN" remove -p hello

echo ""
echo "=== Done! All steps completed successfully ==="
