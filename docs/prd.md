# reg-rs Product Requirements Document

## Overview

reg-rs (pronounced "regress") is a command-line regression testing tool that captures command output and exit codes as "golden" baselines, then compares subsequent runs to detect regressions.

## Current State (v0.1.0)

### Implemented Features
- Create tests capturing stdout, stderr, and exit code
- Run tests against baseline with diff detection
- Report results at 4 verbosity levels (summary, names, failures, diffs)
- Remove tests by pattern
- Web-based status server with file-system monitoring (Axum/Tokio)
- SQLite storage per test (.tdb files)
- External file locking for concurrent access
- Conditional debug output (-d flag)
- File-based logging (-l flag)

### Known Issues

#### High Priority
- **Race condition in test execution** (runner.rs): Read-execute-store sequence is not atomic; concurrent runs can corrupt results
- **Lock contention in status server** (server.rs): Mutex held during filesystem walks and DB reads blocks HTTP handler
- **Panics crash server** (monitor.rs): panic!() on mutex poison kills the entire status server process
- **No command timeout** (process.rs): Hanging test commands block indefinitely and hold DB locks
- **Non-atomic operations** (runner.rs): Clear + store of latest results is two transactions; crash between them loses data

#### Medium Priority
- **Hardcoded data directory** (finder.rs): Tests must live in ./data/ -- TODO to use cwd or make configurable
- **Inconsistent error types**: command.rs uses Box<dyn Error>, lower layers use RegError
- **No exit codes**: reg-rs always returns 0 even when regressions are detected
- **No command execution timing**: Test duration is not measured or stored
- **Empty pattern matches everything**: Undocumented behavior in finder.rs

#### Low Priority
- **Double template render** (queries.rs): Template rendered twice when debug mode is on
- **No validation of test inputs**: Empty test names/commands are accepted
- **Version TBD in status view** (templates/views.rs)

---

## Future Roadmap

### Phase 1: Reliability and Polish

- Fix non-atomic database operations (wrap clear+store in single transaction)
- Add command execution timeout with configurable watchdog
- Replace panics in monitor thread with graceful error recovery
- Reduce lock contention in status server (read state outside lock, update inside)
- Add meaningful exit codes (0=all pass, 1=regressions found, 2=error)
- Make data directory configurable (--data-dir flag or env var)
- Standardize on RegError throughout (remove Box<dyn Error> from public API)

### Phase 2: Dogfooding and Self-Testing

- Create regression tests for reg-rs's own CLI output (help, version, subcommands)
- Generate VHS demo GIFs and link from README.md
- Add self-test Makefile target or script
- Document self-testing workflow as a usage example

### Phase 3: Parallel Execution and Timing

- Run tests in parallel (configurable thread pool)
- Measure and store test execution duration
- Add timeout per test (--timeout flag)
- Store timing data in database (duration, timeout status)
- Report timing information at appropriate verbosity levels

### Phase 4: Smarter Diff and Golden File Management

- Add mechanism to ignore allowed differences (timestamps, durations, paths)
- Regex-based diff exclusion patterns
- Support for ordered vs unordered output comparison
- Recovery mechanism for corrupt or inconsistent test databases

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
