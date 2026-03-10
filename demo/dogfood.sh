#!/bin/bash
# Dogfood script: reg-rs tests itself
# Creates regression tests for reg-rs's own CLI help output,
# then runs and reports on them.
set -e

cd "$(dirname "$0")/.."

echo "=== reg-rs Dogfood: Testing Itself ==="

# Build
echo "Building reg-rs..."
cargo build --release 2>/dev/null

# Ensure data directory exists
mkdir -p data

# Clean previous self-tests
echo ""
echo "=== Cleaning previous self-tests ==="
rm -f data/reg_rs_*.tdb data/reg_rs_*.tdb.lock

# Create self-tests for each command's help output
echo ""
echo "=== Creating self-tests ==="
./target/release/reg-rs create -t data/reg_rs_help.tdb -c './target/release/reg-rs -h'
./target/release/reg-rs create -t data/reg_rs_create_help.tdb -c './target/release/reg-rs create -h'
./target/release/reg-rs create -t data/reg_rs_run_help.tdb -c './target/release/reg-rs run -h'
./target/release/reg-rs create -t data/reg_rs_report_help.tdb -c './target/release/reg-rs report -h'
./target/release/reg-rs create -t data/reg_rs_remove_help.tdb -c './target/release/reg-rs remove -h'
./target/release/reg-rs create -t data/reg_rs_status_help.tdb -c './target/release/reg-rs status -h'

# Run them
echo ""
echo "=== Running self-tests ==="
./target/release/reg-rs run -p reg_rs

# Report results
echo ""
echo "=== Self-test report ==="
./target/release/reg-rs report -p reg_rs -vv

echo ""
echo "=== Done! reg-rs successfully tested itself ==="
