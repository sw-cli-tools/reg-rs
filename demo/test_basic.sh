#!/bin/bash
# Test script for basic demo - run this to verify commands work before VHS recording
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

echo "=== reg-rs Basic Workflow Test ==="

# Ensure data directory exists
mkdir -p "$REG_RS_DATA_DIR"

# Clean up any previous test artifacts
rm -f "$REG_RS_DATA_DIR"/hello.tdb "$REG_RS_DATA_DIR"/hello.tdb.lock \
      "$REG_RS_DATA_DIR"/hello.rgt "$REG_RS_DATA_DIR"/hello.out "$REG_RS_DATA_DIR"/hello.err

# Show help
echo ""
echo "--- Step 0: Show help ---"
"$REG_RS_BIN" --help

# Create a test
echo ""
echo "--- Step 1: Create a test ---"
"$REG_RS_BIN" create -t hello -c 'echo hello world' --desc 'Hello output test'

# List tests
echo ""
echo "--- Step 2: List tests ---"
"$REG_RS_BIN" list -p hello

# Run the test
echo ""
echo "--- Step 3: Run the test ---"
"$REG_RS_BIN" run -p hello

# Show test details
echo ""
echo "--- Step 4: Show test details ---"
"$REG_RS_BIN" show -p hello -v

# Report results and check
echo ""
echo "--- Step 5: Report results ---"
REPORT=$("$REG_RS_BIN" report -p hello -v)
echo "$REPORT"

# Clean up
echo ""
echo "--- Step 6: Remove the test ---"
"$REG_RS_BIN" remove -p hello

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
    echo "=== All steps completed successfully ==="
fi
