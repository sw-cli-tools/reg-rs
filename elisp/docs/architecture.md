# Architecture: my-reg-rs.el

## Overview

```
Emacs Keymap (C-c r <key>)
        |
Interactive command (my/reg-rs-*)
        |
Root detection (my/reg-rs-root)
        |
Shell wrapper (source-rg.sh + alias)
        |
compilation-mode buffer (*reg-rs*)
```

## Layers

### 1. Keybinding Layer

A single sparse keymap `my/reg-rs-map` bound to `C-c r`. Each key dispatches to one interactive command. No prefix-arg logic; commands that need input use `read-string`.

### 2. Command Layer

Thin interactive functions that map to reg-rs shell aliases:

| Function              | Alias  | Purpose              |
|-----------------------|--------|----------------------|
| `my/reg-rs-run-all`   | `rnrg` | Run all tests        |
| `my/reg-rs-run`       | `rnrg` | Run with args/filter |
| `my/reg-rs-list`      | `lsrg` | List tests           |
| `my/reg-rs-show`      | `shrg` | Show test details    |
| `my/reg-rs-update`    | `uprg` | Accept new baseline  |
| `my/reg-rs-reset`     | `rsrg` | Reset results        |
| `my/reg-rs-remove`    | `rmrg` | Delete test          |
| `my/reg-rs-add`       | `adrg` | Create test          |
| `my/reg-rs-rerun`     | (last) | Repeat last command  |

### 3. Execution Engine

`my/reg-rs--run` is the single execution path:

1. Records the command in `my/reg-rs-last-command`
2. Sets `default-directory` to the detected root
3. Wraps the alias command with shell sourcing via `my/reg-rs--shell-command`
4. Calls `compilation-start` with a named buffer

### 4. Root Detection

`my/reg-rs-root` checks three locations in order:

1. Nearest ancestor containing `work/reg-rs/` directory
2. Nearest ancestor containing `.rgt` or `.tdb` files
3. `project.el` project root (or `default-directory`)

This matches reg-rs's own data directory discovery order.

## Design Decisions

- **Aliases over binary**: Sources `bin/source-rg.sh` to use the same `rnrg`/`lsrg` etc. aliases the user trusts in their terminal.
- **compilation-mode over shell**: Ephemeral buffers with no lingering state. Each run is independent.
- **No dependency on pjmai-rs**: The package is fully standalone. `C-c r` is owned entirely by reg-rs.
- **Separate buffers per action**: List/show/update get their own buffer names to avoid clobbering run output.
