# Usage: my-reg-rs.el

## Installation

Add to your Emacs config:

```elisp
(load "~/github/sw-cli-tools/reg-rs/elisp/my-reg-rs.el")
```

Or with `use-package`:

```elisp
(use-package my-reg-rs
  :load-path "~/github/sw-cli-tools/reg-rs/elisp")
```

## Configuration

The defaults should work if reg-rs is installed at `~/github/sw-cli-tools/reg-rs/`. Otherwise, customize:

```elisp
;; Use zsh instead of bash
(setq my/reg-rs-shell "zsh")

;; Custom path to source-rg.sh
(setq my/reg-rs-source "source /path/to/reg-rs/bin/source-rg.sh")
```

## Keybindings

All commands are under `C-c r`:

| Key       | Action                     | When to use                          |
|-----------|----------------------------|--------------------------------------|
| `C-c r r` | Run all tests              | After making changes, quick check    |
| `C-c r f` | Run with filter            | Test a specific pattern              |
| `C-c r v` | Run verbose                | See pass/fail summary per test       |
| `C-c r V` | Run very verbose           | See full diffs for failures          |
| `C-c r l` | List tests                 | See what tests exist and their status|
| `C-c r s` | Show test details          | Inspect a specific test              |
| `C-c r u` | Update baseline            | Accept intentional output changes    |
| `C-c r x` | Reset results              | Clear cached test state              |
| `C-c r d` | Delete test                | Remove a test you no longer need     |
| `C-c r a` | Add test                   | Create a new regression test         |
| `C-c r R` | Rerun last                 | Repeat whatever you just ran         |

## Typical Workflows

### Edit-Test-Fix Loop

```
1. Edit code in a buffer
2. C-c r r          (run all tests)
3. Review failures in *reg-rs* buffer
4. Fix code
5. C-c r R          (rerun last)
```

### Accept Intentional Changes

```
1. C-c r r          (run tests, see failures from intentional change)
2. C-c r V          (verify the diffs are expected)
3. C-c r u          (update baseline to accept new output)
4. C-c r r          (confirm all passing)
```

### Investigate a Specific Test

```
1. C-c r l          (list all tests)
2. C-c r s my_test  (show details for one test)
3. C-c r f my_test -vv  (run just that test with full output)
```

### Create a New Test

```
1. C-c r a "my_command --flag"  (create test from command)
2. C-c r r                      (run to establish baseline)
```

## Per-Project Configuration

Create `.dir-locals.el` in a project root:

```elisp
;; Always run in parallel for this project
((nil . ((eval . (setq-local compile-command
                             (my/reg-rs--shell-command "rnrg --parallel"))))))
```

## Troubleshooting

**Commands fail with "rnrg: command not found"**
- Check that `my/reg-rs-source` points to the correct `source-rg.sh`
- Verify: `bash -lc "source ~/github/sw-cli-tools/reg-rs/bin/source-rg.sh && rnrg"` works in a terminal

**Wrong project root detected**
- The package looks for `work/reg-rs/` directory, then `.rgt`/`.tdb` files
- Ensure your project has one of these markers, or that `project.el` recognizes the root

**Buffer shows old output**
- Each command reuses its named buffer. Run the command again to refresh.
- Use `C-c r R` to quickly rerun the last command.
