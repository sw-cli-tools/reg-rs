#!/bin/bash
# favicon_setup.sh — Create regression tests for favicon
#
# Creates reg-rs regression tests that exercise the favicon image generator
# using three different binary comparison strategies:
#   1. SHA-256 checksum (fast, 64 bytes storage, no visual triage)
#   2. Base64 encoding (slower, ~33KB storage, enables AI visual triage)
#   3. Hybrid: checksum for fast pass/fail + golden file for triage
#
# All approaches capture favicon's stdout/stderr so that new warnings or
# changed messages are detected as regressions. The "Saved:" path is
# normalized via --preprocess since it contains the temp dir PID.
#
# Usage:
#   bash tests/regression/favicon_setup.sh
#
# Environment:
#   REG_RS_BIN      — path to reg-rs binary (default: ./target/debug/reg-rs)
#   FAVICON_BIN     — path to favicon binary (default: ~/github/sw-cli-tools/favicon/components/app/target/debug/favicon)
#   REG_RS_DATA_DIR — where to store .tdb files (default: ./work/reg-rs/favicon-tests)
#
# Error injection:
#   FAVICON_WATERMARK=SAMPLE causes favicon to overlay a red "SAMPLE"
#   watermark, producing different output for the same input.
#   Tests that pass FAVICON_WATERMARK through to the command will detect
#   this as a regression.

set -euo pipefail

REG_RS_BIN="${REG_RS_BIN:-./target/debug/reg-rs}"
FAVICON_BIN="${FAVICON_BIN:-$HOME/github/sw-cli-tools/favicon/components/app/target/debug/favicon}"
export REG_RS_DATA_DIR="${REG_RS_DATA_DIR:-./work/reg-rs/favicon-tests}"
export RUST_LOG="${RUST_LOG:-warn}"
GOLDEN_DIR="$REG_RS_DATA_DIR/golden"

mkdir -p "$REG_RS_DATA_DIR" "$GOLDEN_DIR"

# Verify binaries exist
if [ ! -x "$REG_RS_BIN" ]; then
  echo "ERROR: reg-rs binary not found at $REG_RS_BIN (run: cargo build)" >&2
  exit 1
fi
if [ ! -x "$FAVICON_BIN" ]; then
  echo "ERROR: favicon binary not found at $FAVICON_BIN" >&2
  exit 1
fi

echo "=== favicon Regression Tests ==="
echo "  reg-rs:    $REG_RS_BIN"
echo "  favicon:   $FAVICON_BIN"
echo "  data dir:  $REG_RS_DATA_DIR"
echo "  golden:    $GOLDEN_DIR"
echo ""

# Helper: create a test with optional flags
create_test() {
  local name="$1"
  local cmd="$2"
  shift 2
  echo "  creating: $name"
  "$REG_RS_BIN" create -t "$name" -c "$cmd" "$@" 2>/dev/null
}

F="$FAVICON_BIN"
T="/tmp/favicon-reg-\$\$"
G="$GOLDEN_DIR"

# Preprocess: normalize the temp path in "Saved: /tmp/favicon-reg-12345.png"
NORM_PATH="sed 's|/tmp/favicon-reg-[0-9]*|/tmp/favicon-reg-PID|g'"

# ============================================================
# Text output tests
# ============================================================

echo "--- Creating tests ---"

create_test "favicon_help" \
  "$F --help 2>&1" \
  --timeout 10 \
  --desc "Help text is stable" \
  --expects "Usage line, flag list"

create_test "favicon_version_format" \
  "$F --version 2>&1 | sed 's/[0-9][0-9]*\\.[0-9][0-9]*\\.[0-9][0-9]*/X.Y.Z/g'" \
  --timeout 10 \
  --desc "Version string format" \
  --expects "favicon X.Y.Z with masked numbers"

create_test "favicon_list_symbols" \
  "$F --list-symbols 2>&1 | wc -l | tr -d ' '" \
  --timeout 10 \
  --desc "Symbol count is stable" \
  --expects "Number of available symbols"

# ============================================================
# Binary output tests — SHA-256 checksum
# ============================================================

create_test "favicon_sha_heart_png" \
  "$F -u heart -T --png -o $T.png 2>&1; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Heart PNG checksum is stable (SHA-256)" \
  --expects "Saved: line + same 64-char hex checksum on every run"

create_test "favicon_sha_star_rotated" \
  "$F -u star --rotate-clock-wise 45 --png -o $T.png 2>&1; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Star rotated 45° checksum is stable" \
  --expects "Same SHA-256 across runs"

create_test "favicon_sha_colored" \
  "$F -t A -f FF0000 -b 0000FF --png -o $T.png 2>&1; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Red A on blue background checksum is stable" \
  --expects "Same SHA-256 across runs"

# ============================================================
# Binary output tests — Base64 encoding
# ============================================================

create_test "favicon_b64_heart_png" \
  "$F -u heart -T --png -o $T.png 2>&1; base64 < $T.png; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Heart PNG base64 is stable (full image in test)" \
  --expects "Saved: line + identical base64 — can be decoded for AI visual triage"

create_test "favicon_b64_rocket_png" \
  "$F -u rocket -T --png -o $T.png 2>&1; base64 < $T.png; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Rocket PNG base64 is stable" \
  --expects "Identical base64 — decodable for visual comparison"

# ============================================================
# Binary output tests — Hybrid (checksum + golden file)
# ============================================================

create_test "favicon_hybrid_heart" \
  "$F -u heart -T --png -o $T.png 2>&1; cp $T.png $G/heart.png; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Heart PNG checksum + golden file at $GOLDEN_DIR/heart.png" \
  --expects "SHA-256 matches; golden file available for visual triage"

create_test "favicon_hybrid_dice" \
  "$F -u dice -T --png -o $T.png 2>&1; cp $T.png $G/dice.png; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Dice PNG checksum + golden file at $GOLDEN_DIR/dice.png" \
  --expects "SHA-256 matches; golden file available for visual triage"

# ============================================================
# Error injection tests
# ============================================================

create_test "favicon_inject_heart_sha" \
  "env \${FAVICON_WATERMARK:+FAVICON_WATERMARK=\$FAVICON_WATERMARK} $F -u heart -T --png -o $T.png 2>&1; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Heart checksum — detects FAVICON_WATERMARK injection" \
  --expects "Passes clean, fails with watermark" \
  --flaky-note "Intentionally fails when FAVICON_WATERMARK env var is set"

create_test "favicon_inject_heart_b64" \
  "env \${FAVICON_WATERMARK:+FAVICON_WATERMARK=\$FAVICON_WATERMARK} $F -u heart -T --png -o $T.png 2>&1; base64 < $T.png; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Heart base64 — detects FAVICON_WATERMARK, enables AI visual triage" \
  --expects "Base64 changes when watermarked — decode both for visual diff" \
  --flaky-note "Intentionally fails when FAVICON_WATERMARK env var is set"

# ============================================================
# Run tests and report
# ============================================================

echo ""
echo "--- Running tests ---"
"$REG_RS_BIN" run -p favicon --parallel

echo ""
echo "--- Report ---"
REPORT=$("$REG_RS_BIN" report -p favicon -v)
echo "$REPORT"

FAILED=$(echo "$REPORT" | grep -o '[0-9]* failed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')
PASSED=$(echo "$REPORT" | grep -o '[0-9]* passed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')

echo ""
if [ "${FAILED:-0}" -gt 0 ]; then
    echo "=== FAILED: ${FAILED} favicon test(s) failed ==="
    echo ""
    echo "To see diffs:   REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN show -p favicon -vv"
    echo "To rebase:      REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN rebase -p favicon"
    exit 1
elif [ "${PASSED:-0}" -eq 0 ]; then
    echo "=== ERROR: No tests were run ==="
    exit 2
else
    echo "=== PASSED: All ${PASSED} favicon test(s) passed ==="
fi
