# reg-rs Product Requirements Document

## Overview

reg-rs (pronounced "regress") is a command-line regression testing tool that captures command output and exit codes as "golden" baselines, then compares subsequent runs to detect regressions.

## Current State (v0.1.0)

### Implemented Features
- Create tests capturing stdout, stderr, and exit code
- Run tests against baseline with diff detection
- Report results at 4 verbosity levels (summary, names, failures, diffs)
- Remove tests by pattern
- Parallel test execution (`--parallel` flag)
- Auto-discovery of data directory (`./work/reg-rs/` → cwd → `~/.local/reg-rs/`)
- Optional `-p` pattern (defaults to all tests); tests run in alphabetical order
- Output preprocessing (`--preprocess` flag) and diff modes (text, json, lines-unordered)
- AI-powered test creation (`--describe` flag) and failure analysis (`analyze` subcommand)
- Self-documenting test metadata (`--desc`, `--expects`, `--flaky-note`)
- Web-based status dashboard with real-time SSE updates
  - Landing page with summary counts and status indicator
  - Status dashboard with collapsible sections, colored indicators, nav links
  - Character-level diff highlighting for failed tests
  - Automatic DOM updates via Server-Sent Events (no manual refresh)
  - SSE event counter displayed in footer
  - JSON API endpoint (`/api/status`) for programmatic access
- SQLite storage per test (.tdb files)
- External file locking for concurrent access
- `list` subcommand for quick test enumeration with status
- `bin/regress` wrapper script for ergonomic usage
- Conditional debug output (-d flag)
- File-based logging (-l flag)

### Known Issues

#### High Priority
- **Race condition in test execution** (runner.rs): Read-execute-store sequence is not atomic; concurrent runs can corrupt results
- **Non-atomic operations** (runner.rs): Clear + store of latest results is two transactions; crash between them loses data

#### Medium Priority
- **No `show` subcommand**: Cannot view stored test commands without using sqlite3 directly (see docs/gaps.md)
- **Substring-only pattern matching**: `-p` uses literal substring match, not regex/glob
- **Inconsistent error types**: command.rs uses Box<dyn Error>, lower layers use RegError
- **No exit codes**: reg-rs always returns 0 even when regressions are detected
- **No command execution timing**: Test duration is not measured or stored

#### Low Priority
- **Double template render** (queries.rs): Template rendered twice when debug mode is on
- **No validation of test inputs**: Empty test names/commands are accepted

### Recently Resolved
- ~~Hardcoded data directory~~: Auto-discovery now checks `$REG_RS_DATA_DIR` → `./work/reg-rs/` → cwd → `~/.local/reg-rs/`
- ~~Pattern required on all commands~~: `-p` now defaults to `.tdb` (match all)
- ~~Panics crash server~~: Monitor thread uses graceful error recovery
- ~~No command timeout~~: `--timeout` flag added to process execution
- ~~Lock contention in status server~~: Lock scope reduced, SSE uses broadcast channel
- ~~Unhelpful error messages~~: Data directory path included in "no tests matched" warnings
- ~~No real-time web updates~~: SSE pushes updates, JS swaps DOM without reload

---

## Future Roadmap

### Phase 1: Reliability and Polish *(mostly complete)*

- [x] Add command execution timeout with configurable watchdog
- [x] Replace panics in monitor thread with graceful error recovery
- [x] Reduce lock contention in status server
- [x] Make data directory auto-discoverable
- [x] Default pattern to match all tests
- [ ] Fix non-atomic database operations (wrap clear+store in single transaction)
- [ ] Add meaningful exit codes (0=all pass, 1=regressions found, 2=error)
- [ ] Standardize on RegError throughout (remove Box<dyn Error> from public API)
- [ ] Add `show` subcommand to view stored test commands and metadata
- [x] Add `list` subcommand for quick test enumeration

### Phase 2: Dogfooding and Self-Testing *(complete)*

- [x] Create regression tests for reg-rs's own CLI output
- [x] Generate VHS demo GIFs and link from README.md
- [x] Demo scripts run as part of `cargo test`
- [x] Document self-testing workflow
- [x] Regression test suites for external projects (pjmai-rs, favicon, rank-wav)

### Phase 3: Parallel Execution and Timing *(partially complete)*

- [x] Run tests in parallel (`--parallel` flag)
- [ ] Measure and store test execution duration
- [ ] Store timing data in database (duration, timeout status)
- [ ] Report timing information at appropriate verbosity levels

### Phase 4: Smarter Diff and Golden File Management *(partially complete)*

- [x] Output preprocessing (`--preprocess` flag for external commands)
- [x] Built-in diff modes (text, json key sorting, lines-unordered)
- [ ] Regex-based diff exclusion patterns
- [ ] Recovery mechanism for corrupt or inconsistent test databases

### Phase 5: Web Dashboard *(complete)*

- [x] Landing page with summary counts and status indicator
- [x] Status dashboard with collapsible sections, colored stat cards, nav links
- [x] Character-level diff highlighting (expected vs actual with `<mark>` tags)
- [x] Real-time updates via Server-Sent Events (no manual refresh)
- [x] SSE event counter in footer
- [x] JSON API endpoint (`/api/status`)
- [x] Accessible design (shapes + colors for colorblind users)
- [ ] Yew/WASM frontend (see docs/plan-yew-frontend.md — possible future feature)

---

## AI Agent Integration (Wish List)

### A. Agent-Oriented CLI Help

**Goal**: Make it easier for AI coding agents to discover and use reg-rs effectively.

**Requirements**:
- Add `--help-agent` or `--help-json` flag that outputs structured, machine-readable help
- Include input/output schemas, expected patterns, common workflows in structured format
- Provide example command sequences as structured data (not just prose)
- Output exit code semantics in machine-parseable form
- Include error taxonomy: which errors are retryable, which need human intervention
- Annotate each flag with: type, default, constraints, side effects

**Use cases**:
- AI agent reads `reg-rs --help-agent` and generates correct commands without trial-and-error
- Agent can programmatically determine which flags to use for a given workflow
- Agent can understand error responses and take corrective action

**Design considerations**:
- JSON output format for easy parsing by any agent framework
- Backward-compatible: existing --help unchanged
- Include "recipes" section with common multi-step workflows
- Version the schema so agents can adapt to CLI changes

### B. Intelligent Golden File Management

**Goal**: Help AI agents create better golden reference files by distinguishing stable output from environmental noise.

**Requirements**:
- Allow marking sections of test output as "stable" (must match) vs "volatile" (can change)
- Support patterns for common volatile data:
  - Timestamps and durations (ISO 8601, Unix epoch, human-readable)
  - Absolute paths (replace with relative or placeholder)
  - Process IDs, port numbers, memory addresses
  - UUIDs and random tokens
  - Output ordering that varies between runs
- Provide `reg-rs analyze` command that examines multiple runs and suggests which parts are volatile
- Store volatility annotations alongside golden files
- AI agent can run `reg-rs analyze -p my_test` after N runs to auto-generate masks

**Design considerations**:
- Annotation format: inline markers in golden file, or separate mask/schema file?
- Regex-based masks stored in a `.tdb.mask` sidecar file
- `reg-rs create --smart` could run command N times and auto-detect volatile sections
- Diff engine needs to respect masks: masked sections always "pass"
- Agent workflow: create test -> run N times -> analyze -> apply masks -> stable golden file

**Example workflow**:
```
# Agent creates a test
reg-rs create -t data/build_test.tdb -c 'cargo build 2>&1'

# Agent runs it several times to gather variance data
reg-rs run -p build_test
reg-rs run -p build_test
reg-rs run -p build_test

# Agent asks reg-rs to analyze what changed
reg-rs analyze -p build_test
# Output: "Lines 3,7,12 vary between runs (timestamps, durations)"
# Output: "Suggested masks: line 3: /\d+\.\d+s/, line 7: /202\d-.*/"

# Agent applies masks
reg-rs mask -p build_test --accept-suggestions

# Future runs ignore masked sections
reg-rs run -p build_test  # PASS (volatile sections excluded)
```

### C. AI-Powered Test Failure Interpretation

**Goal**: Help AI agents distinguish real regressions from flaky tests, and make tests more reliable.

**Requirements**:
- Add `reg-rs diagnose -p <pattern>` command that analyzes failure history
- Classify failures into categories:
  - **True regression**: Output changed consistently after a code change
  - **Flaky test**: Output varies randomly between runs (timing, ordering)
  - **Environmental change**: Output changed due to system update (OS, tool version)
  - **Stale baseline**: Golden file is outdated and needs refresh
- Provide confidence score for each classification
- Suggest remediation for each category:
  - True regression: "Investigate code change between commit X and Y"
  - Flaky test: "Add masks for lines N,M or increase tolerance"
  - Environmental: "Re-baseline with `reg-rs create --force`"
  - Stale baseline: "Review and accept new baseline"
- Store failure history (not just latest) for trend analysis
- Expose diagnostics in machine-readable format for agent consumption

**Design considerations**:
- Failure history table in .tdb database (timestamp, diff summary, classification)
- Flakiness detection: run test K times, measure variance
- `reg-rs diagnose --json` for agent consumption
- Integration with `reg-rs analyze` (Phase B) for mask suggestions
- Agent workflow: test fails -> agent runs `diagnose` -> agent decides action

**Flakiness detection algorithm**:
```
1. Run test N times (configurable, default 5)
2. Compare all N outputs pairwise
3. Lines that vary across runs = volatile (likely flaky)
4. Lines that are consistent across N runs but differ from golden = likely regression
5. Lines that match golden in some runs but not others = intermittent issue
6. Report: "Test is X% flaky. Volatile lines: [...]. Stable regressions: [...]"
```

**Agent integration example**:
```
# Agent detects test failure
reg-rs run -p my_test  # FAIL

# Agent diagnoses the failure
reg-rs diagnose -p my_test --json
# {
#   "classification": "flaky",
#   "confidence": 0.85,
#   "volatile_lines": [3, 7],
#   "suggestion": "mask_volatile_lines",
#   "details": "Lines 3,7 contain timestamps that vary between runs"
# }

# Agent takes corrective action
reg-rs mask -p my_test --lines 3,7 --pattern '\d{4}-\d{2}-\d{2}'

# Agent re-runs to confirm fix
reg-rs run -p my_test  # PASS
```

---

## Success Metrics

- **Reliability**: Zero data loss from concurrent operations
- **Usability**: New user can create and run a test in under 60 seconds
- **AI Agent adoption**: Agent can use reg-rs without human help after reading --help-agent
- **Flakiness reduction**: 90% of flaky tests auto-resolved via masks after `analyze`
- **Self-testing**: reg-rs regression tests catch 100% of CLI output changes
