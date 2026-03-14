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

DBG="$COR24_DIR/target/debug/cor24-dbg"
RUN="$COR24_DIR/rust-to-cor24/target/release/cor24-run"
PROGRAMS="$COR24_DIR/tests/programs"
DEMOS="$COR24_DIR/rust-to-cor24/demos"

mkdir -p "$REG_RS_DATA_DIR"

# Verify binaries exist
if [ ! -x "$REG_RS_BIN" ]; then
  echo "ERROR: reg-rs binary not found at $REG_RS_BIN (run: cargo build)" >&2
  exit 1
fi
if [ ! -x "$DBG" ]; then
  echo "ERROR: cor24-dbg not found at $DBG (run: cd $COR24_DIR && cargo build -p cor24-cli)" >&2
  exit 1
fi
if [ ! -x "$RUN" ]; then
  echo "ERROR: cor24-run not found at $RUN (run: cd $COR24_DIR/rust-to-cor24 && cargo build --release)" >&2
  exit 1
fi

echo "Creating cor24-rs regression tests..."
echo "  reg-rs:    $REG_RS_BIN"
echo "  cor24-dbg: $DBG"
echo "  cor24-run: $RUN"
echo "  data dir:  $REG_RS_DATA_DIR"
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

# TEST: Hello World — basic UART output
# EXPECTS: "Hello, World!" in UART buffer, CPU halts after 93 instructions
# FLAKY: No — deterministic execution of fixed program
# FAILURE CAUSES: Emulator instruction execution, UART peripheral, halt detection
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
# EXPECTS: "54321" in UART buffer, CPU halts after 36 instructions
# FLAKY: No — deterministic loop counting
# FAILURE CAUSES: Loop logic, UART write, decrement/branch instructions
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
# EXPECTS: "LLLLL" UART output, LED D2 OFF at end
# FLAKY: No — deterministic I/O register toggling
# FAILURE CAUSES: LED peripheral, I/O memory mapping, UART output
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
# EXPECTS: Disassembly, breakpoint hit, register values, "54321" UART
# FLAKY: No — deterministic debugger interaction
# FAILURE CAUSES: Breakpoint logic, disassembler, register display, continue command
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
# EXPECTS: Stable instruction disassembly for the hello_world program
# FLAKY: No — disassembly is deterministic
# FAILURE CAUSES: Disassembler formatting, instruction decoding
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
# EXPECTS: "1000 iterations" UART output, stops after 1M instruction limit
# FLAKY: No — deterministic execution with fixed instruction limit
# FAILURE CAUSES: Branch instructions, memory operations, instruction counter
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

# ============================================================
# Rust pipeline demos via cor24-run (headless emulator)
# Uses pre-compiled .cor24.s assembly files — no Rust compilation
# needed. Tests the assembler + emulator, not the Rust compiler.
# ============================================================

# TEST: Rust demo_add — arithmetic result in registers
# EXPECTS: r2 = 0x6C (108 = 42+66), 12 instructions, halted
# FLAKY: No — deterministic arithmetic
# FAILURE CAUSES: ALU operations, register file, program loading
create_test "cor24_rust_add" \
  "$RUN --run $DEMOS/demo_add/demo_add.cor24.s --dump --speed 0 --time 5 2>&1" \
  --timeout 15 \
  --desc "Rust pipeline: add 42+66, check register result" \
  --expects "r2=0x6C (108), 12 instructions, halted"

# TEST: Rust demo_uart_hello — UART output from Rust program
# EXPECTS: UART TX log contains "Hello\n", 79 instructions
# FLAKY: No — deterministic UART writes
# FAILURE CAUSES: UART peripheral, Rust runtime init, string handling
create_test "cor24_rust_uart_hello" \
  "$RUN --run $DEMOS/demo_uart_hello/demo_uart_hello.cor24.s --dump --speed 0 --time 5 2>&1" \
  --timeout 15 \
  --desc "Rust pipeline: UART Hello output" \
  --expects "UART TX log shows 'Hello', 79 instructions, halted"

# TEST: Rust demo_fibonacci — recursive computation
# EXPECTS: LED register shows fibonacci result, 4764 instructions
# FLAKY: No — deterministic recursion
# FAILURE CAUSES: Stack operations, function calls, recursion depth
create_test "cor24_rust_fibonacci" \
  "$RUN --run $DEMOS/demo_fibonacci/demo_fibonacci.cor24.s --dump --speed 0 --time 5 2>&1" \
  --timeout 15 \
  --desc "Rust pipeline: recursive Fibonacci computation" \
  --expects "LED shows result (0x59), 4764 instructions, halted"

# TEST: Rust demo_countdown — loop with UART output
# EXPECTS: 80332 instructions, halted, countdown via UART-mapped I/O
# FLAKY: No — deterministic loop execution
# FAILURE CAUSES: Loop control flow, branch instructions, memory-mapped I/O
create_test "cor24_rust_countdown" \
  "$RUN --run $DEMOS/demo_countdown/demo_countdown.cor24.s --dump --speed 0 --time 5 2>&1" \
  --timeout 15 \
  --desc "Rust pipeline: countdown loop" \
  --expects "80332 instructions, halted"

echo ""
echo "Done! Created $(ls "$REG_RS_DATA_DIR"/*.tdb 2>/dev/null | wc -l | tr -d ' ') regression tests"
echo ""
echo "Run:     REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN run -p cor24"
echo "Report:  REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN report -p cor24 -vvv"
echo "Analyze: REG_RS_DATA_DIR=$REG_RS_DATA_DIR $REG_RS_BIN analyze -p cor24"
