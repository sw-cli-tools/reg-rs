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

echo "Creating favicon regression tests..."
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
# so the PID portion doesn't cause false failures.
NORM_PATH="sed 's|/tmp/favicon-reg-[0-9]*|/tmp/favicon-reg-PID|g'"

# ============================================================
# Text output tests (standard reg-rs comparison)
# ============================================================

# TEST: Help text is stable
# EXPECTS: Usage line, flag list
# FLAKY: No
# FAILURE CAUSES: CLI arg change, clap version bump
create_test "favicon_help" \
  "$F --help 2>&1" \
  --timeout 10 \
  --desc "Help text is stable" \
  --expects "Usage line, flag list"

# TEST: Version format
# EXPECTS: "favicon X.Y.Z" with masked version
# FLAKY: No after masking
# FAILURE CAUSES: version format change
create_test "favicon_version_format" \
  "$F --version 2>&1 | sed 's/[0-9][0-9]*\\.[0-9][0-9]*\\.[0-9][0-9]*/X.Y.Z/g'" \
  --timeout 10 \
  --desc "Version string format" \
  --expects "favicon X.Y.Z with masked numbers"

# TEST: Symbol count
# EXPECTS: Known count of symbol aliases
# FLAKY: No
# FAILURE CAUSES: Symbols added/removed
create_test "favicon_list_symbols" \
  "$F --list-symbols 2>&1 | wc -l | tr -d ' '" \
  --timeout 10 \
  --desc "Symbol count is stable" \
  --expects "Number of available symbols"

# ============================================================
# Binary output tests — Approach 1: SHA-256 checksum
# Storage: ~100 bytes per test (64-char checksum + "Saved:" line).
# Fast. Cannot reconstruct image for AI triage.
# Captures favicon stdout so warnings are detected.
# ============================================================

# TEST: Heart emoji PNG checksum
# EXPECTS: Identical SHA-256 + identical "Saved:" message
# FLAKY: No — same Skia version + same fonts = byte-identical
# FAILURE CAUSES: Skia upgrade, font change, rendering logic change, new warning
create_test "favicon_sha_heart_png" \
  "$F -u heart -T --png -o $T.png 2>&1; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Heart PNG checksum is stable (SHA-256)" \
  --expects "Saved: line + same 64-char hex checksum on every run"

# TEST: Rotated star PNG checksum
create_test "favicon_sha_star_rotated" \
  "$F -u star --rotate-clock-wise 45 --png -o $T.png 2>&1; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Star rotated 45° checksum is stable" \
  --expects "Same SHA-256 across runs"

# TEST: Custom colors checksum
create_test "favicon_sha_colored" \
  "$F -t A -f FF0000 -b 0000FF --png -o $T.png 2>&1; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Red A on blue background checksum is stable" \
  --expects "Same SHA-256 across runs"

# ============================================================
# Binary output tests — Approach 2: Base64 encoding
# Storage: ~33KB per test (full image). Slower.
# AI can decode base64 and visually inspect/compare images.
# Captures favicon stdout so warnings are detected.
# ============================================================

# TEST: Heart emoji PNG base64
create_test "favicon_b64_heart_png" \
  "$F -u heart -T --png -o $T.png 2>&1; base64 < $T.png; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Heart PNG base64 is stable (full image in test)" \
  --expects "Saved: line + identical base64 — can be decoded for AI visual triage"

# TEST: Rocket emoji PNG base64
create_test "favicon_b64_rocket_png" \
  "$F -u rocket -T --png -o $T.png 2>&1; base64 < $T.png; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Rocket PNG base64 is stable" \
  --expects "Identical base64 — decodable for visual comparison"

# ============================================================
# Binary output tests — Approach 3: Hybrid (checksum + golden file)
# Storage: ~100 bytes in .tdb + ~25KB golden file on disk.
# Checksum for fast pass/fail. Golden file for AI visual triage.
# ============================================================

# TEST: Heart emoji hybrid (checksum + golden)
create_test "favicon_hybrid_heart" \
  "$F -u heart -T --png -o $T.png 2>&1; cp $T.png $G/heart.png; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Heart PNG checksum + golden file at $GOLDEN_DIR/heart.png" \
  --expects "SHA-256 matches; golden file available for visual triage"

# TEST: Dice emoji hybrid (checksum + golden)
create_test "favicon_hybrid_dice" \
  "$F -u dice -T --png -o $T.png 2>&1; cp $T.png $G/dice.png; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Dice PNG checksum + golden file at $GOLDEN_DIR/dice.png" \
  --expects "SHA-256 matches; golden file available for visual triage"

# ============================================================
# Error injection tests — verify FAVICON_WATERMARK detection
# These tests are EXPECTED to fail when FAVICON_WATERMARK is set.
# They demonstrate that reg-rs detects binary output regressions.
# ============================================================

# TEST: Watermark-aware heart (checksum — passes env through)
# Baseline created clean. When re-run with FAVICON_WATERMARK=SAMPLE,
# the watermark changes the image → different checksum → detected failure.
create_test "favicon_inject_heart_sha" \
  "env \${FAVICON_WATERMARK:+FAVICON_WATERMARK=\$FAVICON_WATERMARK} $F -u heart -T --png -o $T.png 2>&1; shasum -a 256 $T.png | cut -d' ' -f1; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Heart checksum — detects FAVICON_WATERMARK injection" \
  --expects "Passes clean, fails with watermark" \
  --flaky-note "Intentionally fails when FAVICON_WATERMARK env var is set"

# TEST: Watermark-aware heart (base64 — enables AI visual comparison)
# Same as above, but stores full base64 so an AI vision model can
# decode both the golden and failing images and describe the difference.
create_test "favicon_inject_heart_b64" \
  "env \${FAVICON_WATERMARK:+FAVICON_WATERMARK=\$FAVICON_WATERMARK} $F -u heart -T --png -o $T.png 2>&1; base64 < $T.png; rm -f $T.png" \
  --timeout 10 \
  -P "$NORM_PATH" \
  --desc "Heart base64 — detects FAVICON_WATERMARK, enables AI visual triage" \
  --expects "Base64 changes when watermarked — decode both for visual diff" \
  --flaky-note "Intentionally fails when FAVICON_WATERMARK env var is set"

echo ""
echo "Done! Created $(ls "$REG_RS_DATA_DIR"/*.tdb 2>/dev/null | wc -l | tr -d ' ') regression tests"
echo ""
echo "Run (normal):     REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN run -p favicon --parallel"
echo "Run (inject):     FAVICON_WATERMARK=SAMPLE REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN run -p favicon_inject --parallel"
echo "Report:           REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN report -p favicon -vvv"
echo "Golden files:     ls $GOLDEN_DIR/"
