# Design: my-reg-rs.el

## Keybinding Layout

All commands live under `C-c r`:

| Key       | Command                    | Description            |
|-----------|----------------------------|------------------------|
| `C-c r r` | `my/reg-rs-run-all`        | Run all tests          |
| `C-c r f` | `my/reg-rs-run`            | Run with filter/args   |
| `C-c r v` | `my/reg-rs-run-verbose`    | Run verbose (`-v`)     |
| `C-c r V` | `my/reg-rs-run-very-verbose` | Run very verbose (`-vv`) |
| `C-c r l` | `my/reg-rs-list`           | List tests             |
| `C-c r s` | `my/reg-rs-show`           | Show test details      |
| `C-c r u` | `my/reg-rs-update`         | Update/rebase baseline |
| `C-c r x` | `my/reg-rs-reset`          | Reset test results     |
| `C-c r d` | `my/reg-rs-remove`         | Delete/remove test     |
| `C-c r a` | `my/reg-rs-add`            | Add/create test        |
| `C-c r R` | `my/reg-rs-rerun`          | Rerun last command     |

## Shell Execution Model

Commands are executed via:

```
bash -lc "source ~/github/sw-cli-tools/reg-rs/bin/source-rg.sh && rnrg <args>"
```

The `-lc` flag ensures a login shell context so aliases and PATH are available. The source command is configurable via `my/reg-rs-source`.

## Buffer Strategy

| Action          | Buffer Name       |
|-----------------|-------------------|
| Run tests       | `*reg-rs*`        |
| List tests      | `*reg-rs-list*`   |
| Show details    | `*reg-rs-show*`   |
| Update baseline | `*reg-rs-update*` |
| Reset results   | `*reg-rs-reset*`  |
| Remove test     | `*reg-rs-remove*` |
| Add test        | `*reg-rs-add*`    |

Separate buffer names prevent test output from being overwritten by list/show operations.

## Customization Points

| Variable              | Default                                              | Purpose                    |
|-----------------------|------------------------------------------------------|----------------------------|
| `my/reg-rs-shell`     | `"bash"`                                             | Shell for execution        |
| `my/reg-rs-source`    | `"source ~/github/sw-cli-tools/reg-rs/bin/source-rg.sh"` | Alias sourcing command |
| `my/reg-rs-buffer-name` | `"*reg-rs*"`                                      | Default buffer name        |

### Per-Project Customization

Use `.dir-locals.el` to override defaults per project:

```elisp
((nil . ((my/reg-rs-source . "source /custom/path/source-rg.sh"))))
```

### compile-command Integration

For projects dominated by reg-rs testing:

```elisp
(setq-local compile-command (my/reg-rs--shell-command "rnrg"))
```

Then `M-x compile` and `M-x recompile` run regression tests.

## Alias-to-Command Mapping

| Alias  | reg-rs Subcommand | Purpose                         |
|--------|-------------------|---------------------------------|
| `rnrg` | `run`             | Execute tests, report results   |
| `lsrg` | `list`            | Show test names and status      |
| `shrg` | `show`            | Display detailed test info      |
| `uprg` | `rebase`          | Accept current output as new baseline |
| `rsrg` | `reset`           | Clear cached test results       |
| `rmrg` | `remove`          | Delete test definition          |
| `adrg` | `create`          | Create new regression test      |
