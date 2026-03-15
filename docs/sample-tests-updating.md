# Updating Regression Tests After Refactoring

This document walks through the reg-rs workflow for detecting, diagnosing, and resolving test failures after a codebase change — using the cor24 emulator test suite as a real-world example.

## Background

The cor24 test suite (`work/reg-rs/cor24-tests/`) contains 18 regression tests covering:

| Category | Tests | What they capture |
|----------|-------|-------------------|
| Debugger (cor24-dbg) | 6 | UART output, LED state, disassembly, breakpoints |
| Assembler listing | 4 | Address/opcode/mnemonic from `cor24-run --assemble` |
| Translator | 4 | MSP430-to-COR24 assembly output from `msp430-to-cor24` |
| Rust pipeline execution | 4 | Full register/memory dumps from `cor24-run --run` |

## Scenario: cor24-run Rebuilt with Code Generation Changes

After refactoring the cor24 assembler/emulator, the `cor24-run` binary was rebuilt. Running the test suite reveals regressions:

```bash
export REG_RS_DATA_DIR=./work/reg-rs/cor24-tests
reg-rs reset -p cor24
reg-rs run -p cor24 --parallel
```

Output:
```
running 18 tests in parallel
parallel run complete: 18 tests in 0.12s
```

Exit code: **1** (regressions detected).

## Step 1: List test status

```bash
reg-rs list -p cor24
```

Shows which tests passed and which failed. The summary line gives counts.

Using the shell alias: `lsrg -p cor24`

## Step 2: Identify what changed

```bash
reg-rs show -p cor24_rust_add -vv
```

The `show -vv` output displays three sections:
1. **Baseline stdout** — the expected ("golden") output
2. **Latest stdout** — the actual output from the most recent run
3. **Differences** — lines marked `[stdout remove]` (baseline) and `[stdout add]` (actual)

Example diff for the add test:
```
[stdout remove] Assembled 120 bytes
[stdout add] Assembled 104 bytes
...
[stdout remove] Executed 12 instructions
[stdout add] Executed 15 instructions
...
[stdout remove]   Instructions: 12
[stdout add]   Instructions: 15
```

This tells us the assembler now generates smaller code (104 vs 120 bytes) but the CPU needs more instructions (15 vs 12) — likely a code generation optimization that trades code size for instruction count.

Using the alias: `shrg -p cor24_rust_add -vv`

## Step 3: Decide — regression or intentional change?

For each failing test, ask:
- **Is the new output correct?** → Rebase (accept new baseline)
- **Is the new output wrong?** → Fix the code that caused the regression

In this case, the assembler refactoring intentionally changed code generation. The computation results are identical (r2=0x6C=108 for add, fibonacci still produces 0x59=89), so the behavioral outcome is preserved. The instruction counts changed because the assembler generates different (smaller) machine code.

## Step 4: Rebase — accept new output as baseline

```bash
reg-rs rebase -p cor24_rust
```

Output:
```
rebased: .../cor24_rust_add.rgt
rebased: .../cor24_rust_countdown.rgt
rebased: .../cor24_rust_fibonacci.rgt
rebased: .../cor24_rust_uart_hello.rgt
4 test(s) rebased
```

This updates the `.out` baseline files with the latest execution output. The `-p cor24_rust` pattern matches only the 4 Rust pipeline tests.

Using the alias: `uprg -p cor24_rust`

## Step 5: Re-run and verify

```bash
reg-rs reset -p cor24_rust
reg-rs run -p cor24_rust --parallel
reg-rs report -p cor24_rust -v
```

Output:
```
00000 failed
00000 not yet run
00004 passed
```

All 4 rebased tests now pass.

Using aliases: `rnrg -p cor24_rust --parallel`

## Step 6: Handle broken test dependencies

In this scenario, 6 debugger tests also failed — not because of code changes, but because the `cor24-dbg` binary was no longer built (removed from debug target). The test output shows:

```
--- latest stderr ---
sh: .../cor24-dbg: No such file or directory
--- latest exit: 127 ---
```

These tests **cannot be rebased** — rebasing would accept "binary not found" as the new baseline, which is wrong. Instead:
- Rebuild the binary: `cd cor24-rs && cargo build -p cor24-cli`
- Or temporarily skip these tests until the binary is available

## Test Categories and What Changes Affect Them

| Change | Debugger tests | Assembler listing | Translator | Execution |
|--------|---------------|-------------------|------------|-----------|
| Emulator behavior | Yes | No | No | Yes |
| Assembler code gen | No | Yes | No | Yes |
| Translator logic | No | No | Yes | Yes |
| Debugger UI/output | Yes | No | No | No |
| Rust source (demos) | No | No | No | Yes |

Key insight: **Execution tests are the most sensitive** — they catch changes from any layer. **Assembler listing and translator tests are more targeted** — they pinpoint exactly which stage changed. This layered approach makes it easy to diagnose whether a regression came from the translator, the assembler, or the emulator.

## Quick Reference

| Action | Command | Alias |
|--------|---------|-------|
| Run all cor24 tests | `reg-rs run -p cor24 --parallel` | `rnrg -p cor24 --parallel` |
| List test status | `reg-rs list -p cor24` | `lsrg -p cor24` |
| Show diffs | `reg-rs show -p <name> -vv` | `shrg -p <name> -vv` |
| Accept new output | `reg-rs rebase -p <pattern>` | `uprg -p <pattern>` |
| Clear cached results | `reg-rs reset -p <pattern>` | — |
| Full report | `reg-rs report -p cor24 -v` | — |

## Setup

To recreate the full cor24 test suite from scratch:

```bash
bash tests/regression/cor24_setup.sh
```

This creates all 18 tests, migrates to `.rgt` format, runs them, and reports results.
