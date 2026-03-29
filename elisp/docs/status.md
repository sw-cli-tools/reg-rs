# Status: my-reg-rs.el

## Current: v0.1.0

Phase 1 (Core) is complete. The package provides:

- 11 interactive commands covering the full reg-rs workflow
- Keymap under `C-c r` with mnemonic single-letter bindings
- Project-aware root detection matching reg-rs discovery rules
- compilation-mode output with separate named buffers per action
- Rerun-last-command support
- Configurable shell, source path, and buffer name

## What Works

- `C-c r r` runs all tests from project root
- `C-c r f` prompts for filter args
- `C-c r l/s/u/x/d/a` map to list/show/update/reset/remove/add
- `C-c r v/V` for verbose modes
- `C-c r R` reruns the last command
- Root detection finds `work/reg-rs/` or `.rgt`/`.tdb` files

## Not Yet Implemented

- Transient menu UI
- Test name completion
- ANSI color rendering
- `next-error` integration
- Diff viewer integration
