#!/bin/bash
# cor24_setup.sh — Create regression tests for cor24-rs emulator
#
# Creates reg-rs regression tests exercising the COR24 24-bit RISC CPU
# emulator (cor24-dbg debugger and cor24-run headless runner) against
# pre-built test programs and Rust pipeline demos.
#
# Usage:
#   bash tests/regression/cor24_setup.sh
#
# Environment:
#   REG_RS_BIN      — path to reg-rs binary (default: ./target/debug/reg-rs)
#   COR24_DIR       — path to cor24-rs repo (default: ~/github/sw-embed/cor24-rs)
#   REG_RS_DATA_DIR — where to store .tdb files (default: ./work/reg-rs/cor24-tests)

set -euo pipefail

REG_RS_BIN="${REG_RS_BIN:-./target/debug/reg-rs}"
COR24_DIR="${COR24_DIR:-$HOME/github/sw-embed/cor24-rs}"
export REG_RS_DATA_DIR="${REG_RS_DATA_DIR:-./work/reg-rs/cor24-tests}"
export RUST_LOG="${RUST_LOG:-warn}"

DBG="$COR24_DIR/target/debug/cor24-dbg"
RUN="$COR24_DIR/rust-to-cor24/target/debug/cor24-run"
TRANS="$COR24_DIR/rust-to-cor24/target/debug/msp430-to-cor24"
PROGRAMS="$COR24_DIR/tests/programs"
DEMOS="$COR24_DIR/rust-to-cor24/demos"

mkdir -p "$REG_RS_DATA_DIR"

# Verify binaries exist
if [ ! -x "$REG_RS_BIN" ]; then
  echo "ERROR: reg-rs binary not found at $REG_RS_BIN (run: cargo build)" >&2
  exit 1
fi
HAS_DBG=true
if [ ! -x "$DBG" ]; then
  echo "WARNING: cor24-dbg not found at $DBG — debugger tests will be skipped" >&2
  echo "  (to build: cd $COR24_DIR && cargo build -p cor24-cli)" >&2
  HAS_DBG=false
fi
if [ ! -x "$RUN" ]; then
  echo "ERROR: cor24-run not found at $RUN (run: cd $COR24_DIR/rust-to-cor24 && cargo build --release)" >&2
  exit 1
fi
HAS_TRANS=true
if [ ! -x "$TRANS" ]; then
  echo "WARNING: msp430-to-cor24 not found at $TRANS — translator tests will be skipped" >&2
  HAS_TRANS=false
fi

echo "=== cor24-rs Regression Tests ==="
echo "  reg-rs:         $REG_RS_BIN"
echo "  cor24-dbg:      $DBG $([ "$HAS_DBG" = true ] && echo '(found)' || echo '(MISSING)')"
echo "  cor24-run:      $RUN"
echo "  msp430-to-cor24: $TRANS $([ "$HAS_TRANS" = true ] && echo '(found)' || echo '(MISSING)')"
echo "  data dir:       $REG_RS_DATA_DIR"
echo ""

# Helper: create a test with optional flags
create_test() {
  local name="$1"
  local cmd="$2"
  shift 2
  echo "  creating: $name"
  "$REG_RS_BIN" create -t "$name" -c "$cmd" "$@" 2>/dev/null
}

# ============================================================
# Assembler demos via cor24-dbg (piped stdin commands)
# ============================================================

echo "--- Creating tests ---"

if [ "$HAS_DBG" = false ]; then
  echo "  SKIP: debugger tests (cor24-dbg not found)"
else

# TEST: Hello World — basic UART output
create_test "cor24_hello_world" \
  "$DBG $PROGRAMS/hello_world.lgo <<'CMDS'
run 1000
uart
quit
CMDS" \
  --timeout 10 \
  -P "sed 's|$COR24_DIR|<COR24>|g'" \
  --desc "Hello World UART output from assembler program" \
  --expects "UART buffer contains 'Hello, World!' after 93 instructions"

# TEST: Count Down — UART numeric output
create_test "cor24_count_down" \
  "$DBG $PROGRAMS/count_down.lgo <<'CMDS'
run 1000
uart
quit
CMDS" \
  --timeout 10 \
  -P "sed 's|$COR24_DIR|<COR24>|g'" \
  --desc "Count down from 5 to 1 via UART" \
  --expects "UART buffer contains '54321' after 36 instructions"

# TEST: LED Blink — hardware I/O peripherals
create_test "cor24_led_blink" \
  "$DBG $PROGRAMS/led_blink.lgo <<'CMDS'
run 10000
uart
led
quit
CMDS" \
  --timeout 10 \
  -P "sed 's|$COR24_DIR|<COR24>|g'" \
  --desc "LED blink program toggles D2 five times with UART 'L' output" \
  --expects "UART contains 'LLLLL', LED D2 OFF, CPU halted after 1577 instructions"

# TEST: Count Down with breakpoints and stepping
create_test "cor24_debug_session" \
  "$DBG $PROGRAMS/count_down.lgo <<'CMDS'
disas 0 14
break 0x0B
run
info
print r1
continue
delete all
run
uart
quit
CMDS" \
  --timeout 10 \
  -P "sed 's|$COR24_DIR|<COR24>|g'" \
  --desc "Debugger breakpoint, stepping, register inspection" \
  --expects "Disassembly output, breakpoint hit at 0x0B, r1 value, UART '54321'"

# TEST: Disassembly of hello_world
create_test "cor24_disassembly" \
  "$DBG $PROGRAMS/hello_world.lgo <<'CMDS'
disas 0 20
quit
CMDS" \
  --timeout 10 \
  -P "sed 's|$COR24_DIR|<COR24>|g'" \
  --desc "Disassembly output for hello_world program" \
  --expects "20 disassembled instructions with addresses and mnemonics"

# TEST: Sieve of Eratosthenes — compute-intensive benchmark
SIEVE_LGO="$COR24_DIR/docs/research/asld24/sieve.lgo"
if [ -f "$SIEVE_LGO" ]; then
  create_test "cor24_sieve" \
    "$DBG --entry 0x93 $SIEVE_LGO <<'CMDS'
run 1_000_000
uart
quit
CMDS" \
    --timeout 10 \
    -P "sed 's|$COR24_DIR|<COR24>|g'" \
    --desc "Sieve of Eratosthenes benchmark (1M instruction limit)" \
    --expects "UART shows '1000 iterations', stops after 1000000 instructions"
else
  echo "  SKIP: cor24_sieve (sieve.lgo not found at $SIEVE_LGO)"
fi

fi  # end HAS_DBG

# ============================================================
# Assembler listing tests via cor24-run --assemble
# ============================================================

echo ""
echo "--- Creating assembler listing tests ---"

for demo in demo_add demo_countdown demo_fibonacci demo_uart_hello; do
  test_name="cor24_asm_listing_${demo#demo_}"
  s_file="$DEMOS/$demo/$demo.cor24.s"
  if [ -f "$s_file" ]; then
    create_test "$test_name" \
      "$RUN --assemble $s_file /tmp/cor24_test_$$.bin /dev/stdout 2>&1" \
      --desc "Assembler listing for $demo" \
      --expects "Stable address/opcode/mnemonic listing"
  else
    echo "  SKIP: $test_name ($s_file not found)"
  fi
done

# ============================================================
# MSP430-to-COR24 translator tests
# ============================================================

if [ "$HAS_TRANS" = true ]; then
  echo ""
  echo "--- Creating translator tests ---"

  for demo in demo_add demo_countdown demo_fibonacci demo_uart_hello; do
    test_name="cor24_translate_${demo#demo_}"
    msp_file="$DEMOS/$demo/$demo.msp430.s"
    if [ -f "$msp_file" ]; then
      create_test "$test_name" \
        "$TRANS $msp_file 2>&1" \
        --desc "MSP430-to-COR24 translator output for $demo" \
        --expects "Stable COR24 assembly translation from MSP430 source"
    else
      echo "  SKIP: $test_name ($msp_file not found)"
    fi
  done
else
  echo ""
  echo "  SKIP: translator tests (msp430-to-cor24 not found)"
fi

# ============================================================
# Rust pipeline demos via cor24-run (headless emulator)
# ============================================================

# TEST: Rust demo_add — arithmetic result in registers
create_test "cor24_rust_add" \
  "$RUN --run $DEMOS/demo_add/demo_add.cor24.s --dump --speed 0 --time 5 2>&1" \
  --timeout 15 \
  --desc "Rust pipeline: add 42+66, check register result" \
  --expects "r2=0x6C (108), 12 instructions, halted"

# TEST: Rust demo_uart_hello — UART output from Rust program
create_test "cor24_rust_uart_hello" \
  "$RUN --run $DEMOS/demo_uart_hello/demo_uart_hello.cor24.s --dump --speed 0 --time 5 2>&1" \
  --timeout 15 \
  --desc "Rust pipeline: UART Hello output" \
  --expects "UART TX log shows 'Hello', 79 instructions, halted"

# TEST: Rust demo_fibonacci — recursive computation
create_test "cor24_rust_fibonacci" \
  "$RUN --run $DEMOS/demo_fibonacci/demo_fibonacci.cor24.s --dump --speed 0 --time 5 2>&1" \
  --timeout 15 \
  --desc "Rust pipeline: recursive Fibonacci computation" \
  --expects "LED shows result (0x59), 4764 instructions, halted"

# TEST: Rust demo_countdown — loop with UART output
create_test "cor24_rust_countdown" \
  "$RUN --run $DEMOS/demo_countdown/demo_countdown.cor24.s --dump --speed 0 --time 5 2>&1" \
  --timeout 15 \
  --desc "Rust pipeline: countdown loop" \
  --expects "80332 instructions, halted"

# ============================================================
# Run tests
# ============================================================

echo ""
echo "--- Running tests ---"
"$REG_RS_BIN" run -p cor24

echo ""
echo "--- Report ---"
REPORT=$("$REG_RS_BIN" report -p cor24 -v)
echo "$REPORT"

FAILED=$(echo "$REPORT" | grep -o '[0-9]* failed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')
PASSED=$(echo "$REPORT" | grep -o '[0-9]* passed' | head -1 | grep -o '[0-9]*' | sed 's/^0*//')

echo ""
if [ "${FAILED:-0}" -gt 0 ]; then
    echo "=== FAILED: ${FAILED} cor24 test(s) failed ==="
    echo ""
    echo "To see diffs:   REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN show -p cor24 -vv"
    echo "To rebase:      REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN rebase -p cor24"
    exit 1
elif [ "${PASSED:-0}" -eq 0 ]; then
    echo "=== ERROR: No tests were run ==="
    exit 2
else
    echo "=== PASSED: All ${PASSED} cor24 test(s) passed ==="
fi
