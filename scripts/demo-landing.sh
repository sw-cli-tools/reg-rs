#!/usr/bin/env bash
# demo-landing.sh — Reset tests, start status server, run tests with staggered delays
#
# Shows the landing page (http://localhost:4740/) auto-updating via SSE
# as tests complete one by one.
#
# Usage:
#   bash scripts/demo-landing.sh
#   # Then open http://localhost:4740/ in a browser or OBS capture
#   # Tests will run with 1-second delays, updating the page live

set -euo pipefail
cd "$(dirname "$0")/.."

BIN=components/app/target/release/reg-rs
DATA_DIR=work/reg-rs
export REG_RS_DATA_DIR="$DATA_DIR"

if [ ! -x "$BIN" ]; then
    echo "Building reg-rs..."
    cd components/app && cargo build --release && cd ../..
fi

# Kill any existing status server
pkill -f "reg-rs.*status" 2>/dev/null || true
sleep 1

# Reset all test results so they show as "pending"
echo "Resetting all test results..."
"$BIN" reset -p .rgt 2>/dev/null || true

# Start status server
echo "Starting status server on http://localhost:4740/ ..."
"$BIN" status -p .rgt -l 4740 > /dev/null 2>&1 &
SERVER_PID=$!
sleep 2

echo ""
echo "=== Server running (PID $SERVER_PID) ==="
echo "=== Open http://localhost:4740/ to watch live updates ==="
echo ""
echo "Running tests with 1-second delays..."

# Get list of all test files
TESTS=$("$BIN" list -p .rgt 2>/dev/null | grep -v "^---" | awk '{print $2}')

for test in $TESTS; do
    echo "  Running: $test"
    "$BIN" run -p "$test" -q 2>/dev/null || true
    sleep 1
done

echo ""
echo "=== All tests complete ==="
echo "=== Server still running at http://localhost:4740/ (kill with: kill $SERVER_PID) ==="
