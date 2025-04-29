# Regression Test Tool (RTT1)

A command-line utility that creates, discovers, runs, and reports on regression tests.

## Features

- Create tests that capture command output and exit codes
- Run tests against matching patterns
- Report test results with varying verbosity
- Track and analyze differences between original and latest test runs
- Web-based status monitoring for long-running tests

## Usage

```bash
# Create a new test
rtt1 create -t test_name -c "command to test"

# Run tests matching a pattern
rtt1 run -p pattern

# Report test results
rtt1 report -p pattern -v   # -v, -vv, -vvv for increasing verbosity

# Remove tests
rtt1 remove -p pattern

# Start status monitoring server
rtt1 status -p pattern
```
