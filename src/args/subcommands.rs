#![allow(missing_docs)]

use clap::Parser;

/// Subcommands for reg-rs
#[derive(Debug, PartialEq, Parser)]
pub enum Subcommands {
    /// Create a new test
    #[clap(
        name = "create",
        alias = "c",
        about = "Create a new test",
        long_about = "Creates a new test by executing a command and storing its output.

The command's stdout, stderr, and exit code are captured and stored in a
SQLite database file. This becomes the baseline for future regression tests.

Use --command to specify an explicit shell command, or --describe to have
an AI generate the command from a natural language description.
Requires ANTHROPIC_API_KEY environment variable when using --describe.

EXAMPLES:
  reg-rs create -t pwd_test -c 'pwd'
  reg-rs create -t version -c 'git --version'
  reg-rs c -t ls_test -c 'ls -la'     # using alias
  reg-rs create -t status -D 'show git status of current directory'"
    )]
    Create {
        /// Test name
        #[clap(long, short)]
        test: String,

        /// Shell command to execute
        #[clap(long, short, required_unless_present = "describe")]
        command: Option<String>,

        /// AI-generated command description
        #[clap(long, short = 'D', conflicts_with = "command")]
        describe: Option<String>,

        /// Context command for AI generation
        #[clap(long, short = 'C', requires = "describe")]
        context: Option<String>,

        /// Preprocessing command
        #[clap(long, short = 'P')]
        preprocess: Option<String>,

        /// Diff normalization mode
        #[clap(long, short = 'M', default_value = "text")]
        diff_mode: String,

        /// Command timeout in seconds
        #[clap(long, default_value = "300")]
        timeout: u64,

        /// Test description
        #[clap(long)]
        desc: Option<String>,

        /// Expected behavior
        #[clap(long)]
        expects: Option<String>,

        /// Flakiness notes
        #[clap(long)]
        flaky_note: Option<String>,
    },

    /// Analyze a test failure using AI
    #[clap(
        name = "analyze",
        alias = "a",
        about = "Analyze a test failure using AI",
        long_about = "Uses AI to analyze a test failure and determine root cause.

Sends the test's original output, latest output, and diff to the Claude API
for analysis. The AI determines if the failure is a real regression, a flaky
test, or a baseline that needs updating.

Requires ANTHROPIC_API_KEY environment variable.

EXAMPLES:
  reg-rs analyze -p my_test
  reg-rs a -p failing_test"
    )]
    Analyze {
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },

    /// Shows detailed information about a test
    #[clap(
        name = "show",
        alias = "w",
        about = "Shows detailed information about a test",
        long_about = "Shows detailed information about a test or tests.

Displays the test command, metadata, baseline output, and latest run results.
Use -v for more detail (baseline content), -vv for latest results and diffs.

EXAMPLES:
  reg-rs show -p my_test             # show command and metadata
  reg-rs show -p my_test -v          # also show baseline output
  reg-rs show -p my_test -vv         # also show latest results and diffs
  reg-rs w -p test                   # using alias"
    )]
    Show {
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,

        #[clap(short, action = clap::ArgAction::Count)]
        verbosity: u8,
    },

    /// Lists all tests matching a pattern
    #[clap(
        name = "list",
        alias = "l",
        about = "Lists all tests matching a pattern",
        long_about = "Lists all tests matching a pattern with their name, command, and status.

STATUS VALUES:
  PASS     - Latest run matched baseline
  FAIL     - Latest run differs from baseline
  pending  - Test created but never run

EXAMPLES:
  reg-rs list                        # list all tests
  reg-rs list -p my_test             # list matching tests
  reg-rs l -p pjmai                  # using alias"
    )]
    List {
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },

    /// Outputs test names for shell tab-completion
    #[clap(
        name = "complete",
        about = "Outputs test names for shell tab-completion",
        long_about = "Outputs test names for shell tab-completion.

Prints one test name per line, optionally filtered by a prefix.
Designed to be called by shell completion functions.

EXAMPLES:
  reg-rs complete                      # all test names
  reg-rs complete -p hello             # names matching prefix"
    )]
    Complete {
        #[clap(long, short, default_value = ".rgt")]
        pattern: String,
    },

    /// Clears latest run results
    #[clap(
        name = "reset",
        about = "Clears latest run results",
        long_about = "Clears the latest run results from the .tdb cache for matching tests.

The test baseline (original results or .out/.err files) is preserved.
This effectively marks tests as 'pending' again — useful when you want
to re-run tests from a clean state without recreating them.

EXAMPLES:
  reg-rs reset -p my_test              # reset matching tests
  reg-rs reset                         # reset all tests"
    )]
    Reset {
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },

    /// Converts existing .tdb test databases to .rgt format
    #[clap(
        name = "migrate",
        alias = "m",
        about = "Converts existing .tdb test databases to .rgt format",
        long_about = "Converts existing .tdb test databases to .rgt text format.

For each .tdb file, extracts of test spec (command, timeout, preprocess,
diff_mode, exit_code, metadata) into a TOML .rgt file and writes the
baseline stdout/stderr to .out/.err companion files.

The .tdb file is kept as a runtime cache (.gitignore it).
The .rgt, .out, and .err files are git-friendly text files.

EXAMPLES:
  reg-rs migrate                       # migrate all tests
  reg-rs migrate -p my_test            # migrate matching tests
  reg-rs m -p old_test                 # using alias"
    )]
    Migrate {
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },

    /// Accepts the latest test output as new baseline
    #[clap(
        name = "rebase",
        alias = "u",
        about = "Accepts the latest test output as new baseline",
        long_about = "Accepts the latest test output as the new expected baseline.

For .rgt tests, updates the .out and .err companion files with the latest
run's stdout and stderr. The .rgt spec file is not modified.

For .tdb-only tests, replaces the original results with the latest results.

This is useful when a test output change is intentional (e.g., version bump).

EXAMPLES:
  reg-rs rebase -p my_test             # accept latest output for matching tests
  reg-rs u -p version                  # using alias"
    )]
    Rebase {
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },

    /// Removes test database files matching a pattern
    #[clap(
        name = "remove",
        about = "Removes test database files matching a pattern",
        long_about = "Removes test database files matching a specified pattern.

WARNING: This permanently deletes the test and all stored results!

EXAMPLES:
  reg-rs remove -p old_test
  reg-rs remove -p 'temp_'"
    )]
    Remove {
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },

    /// Reports on test results
    #[clap(
        name = "report",
        alias = "p",
        about = "Reports on test results",
        long_about = "Reports on test results with configurable verbosity.

VERBOSITY LEVELS:
  -q      - Quiet: only exit code (0=pass, 1=fail)
  (none)  - Show only summary counts
  -v      - Also show test names
  -vv     - Also show failure information
  -vvv    - Also show detailed differences

EXAMPLES:
  reg-rs report -p my_test            # basic summary
  reg-rs report -p my_test -v         # show names
  reg-rs report -q -p my_test         # silent, check exit code only
  reg-rs p -p my_test -vvv            # full details (using alias)"
    )]
    Report {
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,

        #[clap(short, action = clap::ArgAction::Count)]
        verbosity: u8,

        #[clap(short, long)]
        quiet: bool,
    },

    /// Runs previously created tests
    #[clap(
        name = "run",
        alias = "r",
        about = "Runs previously created tests",
        long_about = "Runs previously created tests and compares results against baselines.

Each matching test's command is re-executed, and the new output is compared
against the stored baseline. Any differences are recorded as potential regressions.

VERBOSITY LEVELS:
  -q      - Quiet: only exit code (0=pass, 1=fail)
  (none)  - Summary line with pass/fail counts and failing test names
  -v      - Also show failure details (difference counts and types)
  -vv     - Also show full diffs for each failure

EXAMPLES:
  reg-rs run -p my_test                    # run a specific test
  reg-rs run -p test                       # run all matching tests
  reg-rs run -p test -v                    # run with failure details
  reg-rs run -p test -vv                   # run with full diffs
  reg-rs run -q -p test                    # silent, check exit code only
  reg-rs r -p my_test -n                   # dry-run (show what would run)
  reg-rs r -p test --parallel              # run all matching tests in parallel"
    )]
    Run {
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,

        #[clap(long, short = 'n')]
        dry_run: bool,

        #[clap(long)]
        parallel: bool,

        #[clap(short, action = clap::ArgAction::Count)]
        verbosity: u8,

        #[clap(short, long)]
        quiet: bool,
    },

    /// Starts a web server to monitor test results
    #[clap(
        name = "status",
        alias = "s",
        about = "Starts a web server to monitor test results",
        long_about = "Starts a web server to monitor test results in real-time.

The status page shows test counts, pass/fail status, and detailed differences.
The page auto-updates when test files change.

Open http://localhost:<port> in a browser to view the status page.

EXAMPLES:
  reg-rs status -p my_test                 # start on default port 4740
  reg-rs status -p my_test -l 8080         # use custom port
  reg-rs s -p test                         # using alias"
    )]
    Status {
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,

        #[clap(default_value = "4740", long, short)]
        localhost_port: u16,
    },
}
