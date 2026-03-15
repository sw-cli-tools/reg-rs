#!/usr/bin/env bash
# source-rg.sh — Shell aliases for reg-rs
# Source this file from .bashrc or .zshrc:
#   source /path/to/reg-rs/bin/source-rg.sh

# Run tests matching pattern (default: all)
rnrg() { reg-rs run -p "${1:-.rgt}" "${@:2}"; }

# Add (create) a test
adrg() { reg-rs create -t "$1" -c "$2" "${@:3}"; }

# List tests matching pattern (default: all)
lsrg() { reg-rs list -p "${1:-.rgt}"; }

# Show test details
shrg() { reg-rs show -p "$1" "${@:2}"; }

# Start status server
strg() { reg-rs status -p "${1:-.rgt}" "${@:2}"; }

# Reset test results (clear .tdb cache)
rsrg() { reg-rs reset -p "$1"; }

# Update/rebase — accept latest output as new baseline
uprg() { reg-rs rebase -p "$1"; }

# Remove test files
rmrg() { reg-rs remove -p "$1"; }

# Migrate .tdb to .rgt format
mgrg() { reg-rs migrate -p "${1:-.tdb}"; }

# Help — show alias reference
hlrg() {
    cat <<'HELP'
reg-rs shell aliases:
  adrg <name> '<cmd>'   Add/create a test (.rgt + .out baseline)
  rnrg [pattern]        Run tests (default: all)
    rnrg                  run all tests
    rnrg foo              run tests matching "foo"
    rnrg foo -v           run with failure details
    rnrg foo -vv          run with full diffs
    rnrg foo -q           quiet (exit code only)
    rnrg foo --parallel   run in parallel
  lsrg [pattern]        List tests with status
  shrg <name> [-v|-vv]  Show test details/baselines/diffs
  uprg <pattern>        Rebase — accept latest as baseline
  rsrg <pattern>        Reset test results (clear .tdb cache)
  rmrg <pattern>        Remove test files
  strg [pattern]        Start status server
  mgrg [pattern]        Migrate legacy .tdb to .rgt format
  hlrg                  Show this help

Install: source /path/to/reg-rs/bin/source-rg.sh in .zshrc/.bashrc
HELP
}

# --- Shell completion ---

if [ -n "$ZSH_VERSION" ] && type compdef >/dev/null 2>&1; then
    _rg_complete() {
        local completions
        completions=($(reg-rs complete -p "${words[CURRENT]}" 2>/dev/null))
        compadd -a completions
    }
    compdef _rg_complete rnrg lsrg shrg strg rsrg uprg rmrg mgrg
elif [ -n "$BASH_VERSION" ]; then
    _rg_complete() {
        local cur="${COMP_WORDS[COMP_CWORD]}"
        COMPREPLY=($(reg-rs complete -p "$cur" 2>/dev/null))
    }
    complete -F _rg_complete rnrg lsrg shrg strg rsrg uprg rmrg mgrg
fi
