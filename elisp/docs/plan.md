# Implementation Plan: my-reg-rs.el

## Phase 1: Core (Complete)

- [x] Define customization group and variables
- [x] Implement root detection (`my/reg-rs-root`)
- [x] Implement shell command wrapper (`my/reg-rs--shell-command`)
- [x] Implement execution engine (`my/reg-rs--run`)
- [x] Implement all interactive commands (run, list, show, update, reset, remove, add, rerun)
- [x] Implement verbose variants (`-v`, `-vv`)
- [x] Set up keymap under `C-c r`
- [x] Write package header and provide form

## Phase 2: Polish (Future)

- [ ] Add `which-key` descriptions for discoverability
- [ ] Add transient menu as alternative to flat keymap
- [ ] Add compile-command integration helper function
- [ ] Add test name completion (read test names from `lsrg` output)
- [ ] Add ANSI color support in compilation buffer (comint filter)

## Phase 3: Advanced (Future)

- [ ] Parse reg-rs output for `next-error` navigation (compilation-error-regexp)
- [ ] Integration with diff-mode for viewing `.rgt` baseline changes
- [ ] Smart filter based on current buffer (infer which test covers edited file)
- [ ] Parallel execution toggle (`--parallel` flag)
- [ ] Status server integration (open browser to `reg-rs status` dashboard)

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Aliases not loading in non-interactive shell | Use `bash -lc` for login shell; source `bin/source-rg.sh` directly |
| Monorepo root detection fails | `my/reg-rs-root` checks `work/reg-rs/` and `.rgt`/`.tdb` files before fallback |
| Shell environment differences | `my/reg-rs-source` is configurable per-project via `.dir-locals.el` |
