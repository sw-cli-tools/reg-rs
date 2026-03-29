# Product Requirements Document: my-reg-rs.el

## Problem

Running reg-rs regression tests requires switching to a terminal, navigating to the project root, and invoking shell aliases. This context switch interrupts the edit-test-fix cycle.

## Goal

Provide Emacs keybindings that run reg-rs commands from any buffer in a project, displaying results in a compilation buffer with zero configuration beyond loading the package.

## Users

Emacs users who use reg-rs across multiple CLI projects.

## Requirements

### Must Have

- Run all tests with a single keybinding (`C-c r r`)
- Run filtered tests with a prompt (`C-c r f`)
- List, show, update, reset, remove, and add tests via keybindings
- Rerun the last command (`C-c r R`)
- Auto-detect project root using `project.el` and reg-rs directory conventions
- Display output in compilation-mode buffers
- Source `bin/source-rg.sh` to use the documented shell aliases

### Nice to Have

- Verbose (`-v`) and very verbose (`-vv`) run shortcuts
- Per-project default args via `.dir-locals.el`
- `compile-command` integration for `M-x recompile`

### Out of Scope

- Transient/hydra menu (future enhancement)
- Interactive shell buffer mode (pjmai-rs handles shell workflows)
- Parsing reg-rs output for `next-error` navigation
- Integration with pjmai-rs (packages are independent)

## Success Criteria

- Pressing `C-c r r` from any buffer in a project runs all `.rgt` tests and shows results
- The workflow `edit code -> C-c r r -> C-c r u -> C-c r R` takes under 5 keystrokes total
