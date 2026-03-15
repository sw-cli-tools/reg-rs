use std::env;

use clap::Parser;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

/// Argument processing configuration
#[derive(Debug, PartialEq, Parser)]
#[clap(
    name = "reg-rs",
    version = generated_version(),
    long_about = "reg-rs (regress) - A CLI for regression testing

reg-rs captures command output and exit codes as baseline 'golden' results,
then compares subsequent runs against these baselines to detect regressions.

Tests are stored in ~/.local/reg-rs/ by default (override with REG_RS_DATA_DIR).

WORKFLOW:
  1. Create a test:   reg-rs create -t my_test -c 'my_command'
  2. Run the test:    reg-rs run -p my_test
  3. View results:    reg-rs report -p my_test -v

SHELL ALIASES (source bin/source-rg.sh in your .zshrc/.bashrc):
  adrg <name> '<cmd>'   Create a test        (reg-rs create)
  rnrg [pattern]        Run tests            (reg-rs run)
  lsrg [pattern]        List tests           (reg-rs list)
  shrg <name> [-v|-vv]  Show test details    (reg-rs show)
  uprg <pattern>        Rebase baseline      (reg-rs rebase)
  rmrg <pattern>        Remove tests         (reg-rs remove)
  hlrg                  Show alias help

For more information on a specific command, run:
  reg-rs <command> --help"
)]
pub struct Args {
    #[clap(long, short)]
    /// Prints debugging info.  -d must preceed subcommands
    pub debug: bool,
    #[clap(long, short)]
    /// Logs to a log file.  -l must preceed subcommands
    pub logging: bool,
    /// Subcommands
    #[clap(subcommand)]
    pub command: Subcommands,
}

/// Subcommands for reg-rs
#[derive(Debug, PartialEq, Parser)]
pub enum Subcommands {
    /// Creates a new test of a specified command (alias c)
    #[clap(
        name = "create",
        alias = "c",
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
        /// Test name (stored in ~/.local/reg-rs/ as name.tdb)
        #[clap(long, short)]
        test: String,
        /// Shell command to execute and capture (e.g., 'echo hello')
        #[clap(long, short, required_unless_present = "describe")]
        command: Option<String>,
        /// Natural language description — AI generates the command (requires ANTHROPIC_API_KEY)
        #[clap(long, short = 'D', conflicts_with = "command")]
        describe: Option<String>,
        /// Shell command whose output provides context for AI generation (e.g., 'mytool --help')
        #[clap(long, short = 'C', requires = "describe")]
        context: Option<String>,
        /// Shell command to preprocess stdout/stderr before diffing (e.g., "jq --sort-keys")
        #[clap(long, short = 'P')]
        preprocess: Option<String>,
        /// Built-in diff normalization mode: text (default), json (sorts keys), or lines-unordered (sorts lines)
        #[clap(long, short = 'M', default_value = "text")]
        diff_mode: String,
        /// Command timeout in seconds (default: 300)
        #[clap(long, default_value = "300")]
        timeout: u64,
        /// Human-readable description of what this test verifies
        #[clap(long)]
        desc: Option<String>,
        /// Expected behavior or output characteristics
        #[clap(long)]
        expects: Option<String>,
        /// Known flakiness or environmental sensitivity notes
        #[clap(long)]
        flaky_note: Option<String>,
    },
    /// Uses AI to analyze a test failure and determine if it's a real regression or flaky (alias a)
    #[clap(
        name = "analyze",
        alias = "a",
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
        /// Substring pattern to match test names to analyze (default: all)
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },
    /// Shows detailed information about a test including command, baseline, and results (alias w)
    #[clap(
        name = "show",
        alias = "w",
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
        /// Substring pattern to match test names (default: all)
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
        /// Verbosity: -v adds baseline, -vv adds latest results and diffs
        #[clap(short, action = clap::ArgAction::Count)]
        verbosity: u8,
    },
    /// Lists tests matching a pattern with their command and status (alias l)
    #[clap(
        name = "list",
        alias = "l",
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
        /// Substring pattern to match test names (default: all)
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },
    /// Outputs test names for shell completion
    #[clap(
        name = "complete",
        long_about = "Outputs test names for shell tab-completion.

Prints one test name per line, optionally filtered by a prefix.
Designed to be called by shell completion functions.

EXAMPLES:
  reg-rs complete                      # all test names
  reg-rs complete -p hello             # names matching prefix"
    )]
    Complete {
        /// Prefix to filter test names (default: all)
        #[clap(long, short, default_value = ".rgt")]
        pattern: String,
    },
    /// Clears latest results from .tdb cache, keeping baselines intact
    #[clap(
        name = "reset",
        long_about = "Clears the latest run results from the .tdb cache for matching tests.

The test baseline (original results or .out/.err files) is preserved.
This effectively marks tests as 'pending' again — useful when you want
to re-run tests from a clean state without recreating them.

EXAMPLES:
  reg-rs reset -p my_test              # reset matching tests
  reg-rs reset                         # reset all tests"
    )]
    Reset {
        /// Substring pattern to match test names to reset (default: all)
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },
    /// Converts .tdb tests to .rgt text format with .out/.err baselines (alias m)
    #[clap(
        name = "migrate",
        alias = "m",
        long_about = "Converts existing .tdb test databases to .rgt text format.

For each .tdb file, extracts the test spec (command, timeout, preprocess,
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
        /// Substring pattern to match test names to migrate (default: all)
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },
    /// Accepts latest test output as the new baseline, updating .out/.err files (alias u)
    #[clap(
        name = "rebase",
        alias = "u",
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
        /// Substring pattern to match test names to rebase (default: all)
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },
    /// Removes previously created test and run results if any.  Discards test and results!
    #[clap(
        long_about = "Removes test database files matching the specified pattern.

WARNING: This permanently deletes the test and all stored results!

EXAMPLES:
  reg-rs remove -p old_test
  reg-rs remove -p 'temp_'"
    )]
    Remove {
        /// Substring pattern to match test names to remove (default: all)
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
    },
    /// Reports counts/summary of specified test(s) (alias p)
    #[clap(
        name = "report",
        alias = "p",
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
        /// Substring pattern to match test names (default: all)
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
        /// Verbosity: -v adds names, -vv adds failures, -vvv adds differences
        #[clap(short, action = clap::ArgAction::Count)]
        verbosity: u8,
        /// Quiet mode: suppress all output, only set exit code
        #[clap(short, long)]
        quiet: bool,
    },
    /// Runs a test (or tests) based on a test name pattern (alias r)
    #[clap(
        name = "run",
        alias = "r",
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
        /// Substring pattern to match test names to run (default: all)
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
        /// Show what would be run without actually executing
        #[clap(long, short = 'n')]
        dry_run: bool,
        /// Run tests in parallel using one thread per test
        #[clap(long)]
        parallel: bool,
        /// Verbosity: -v adds failure details, -vv adds full diffs
        #[clap(short, action = clap::ArgAction::Count)]
        verbosity: u8,
        /// Quiet mode: suppress all output, only set exit code
        #[clap(short, long)]
        quiet: bool,
    },
    /// Starts a server to monitor long running tests and/or show results (alias s)
    #[clap(
        name = "status",
        alias = "s",
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
        /// Substring pattern to match test names to monitor (default: all)
        #[clap(long, short, default_value = ".tdb")]
        pattern: String,
        /// Port number for the web server (default: 4740)
        #[clap(default_value = "4740", long, short)]
        localhost_port: u16,
    },
}
/// Parse arguments
pub fn parse_args() -> Args {
    Args::parse()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Create {
                    test: "pat001".to_string(),
                    command: Some("pwd".to_string()),
                    describe: None,
                    context: None,
                    preprocess: None,
                    diff_mode: "text".to_string(),
                    timeout: 300,
                    desc: None,
                    expects: None,
                    flaky_note: None,
                },
                debug: false,
                logging: false,
            },
            Args::try_parse_from(["test", "create", "-t", "pat001", "-c", "pwd"]).unwrap()
        );
    }

    #[test]
    fn test_create_no_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Create {
                    test: "pat001".to_string(),
                    command: Some("pwd".to_string()),
                    describe: None,
                    context: None,
                    preprocess: None,
                    diff_mode: "text".to_string(),
                    timeout: 300,
                    desc: None,
                    expects: None,
                    flaky_note: None,
                },
                debug: true,
                logging: false,
            },
            Args::try_parse_from(["test", "-d", "create", "-t", "pat001", "-c", "pwd"]).unwrap()
        );
    }

    #[test]
    fn test_remove_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Remove {
                    pattern: "pat001".to_string(),
                },
                debug: false,
                logging: false,
            },
            Args::try_parse_from(["test", "remove", "-p", "pat001"]).unwrap()
        );
    }

    #[test]
    fn test_remove_no_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Remove {
                    pattern: "pat001".to_string(),
                },
                debug: true,
                logging: false,
            },
            Args::try_parse_from(["test", "-d", "remove", "-p", "pat001"]).unwrap()
        );
    }

    #[test]
    fn test_report_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Report {
                    pattern: "pat001".to_string(),
                    verbosity: 0,
                    quiet: false,
                },
                debug: false,
                logging: false,
            },
            Args::try_parse_from(["test", "report", "-p", "pat001"]).unwrap()
        );
    }

    #[test]
    fn test_report_no_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Report {
                    pattern: "pat001".to_string(),
                    verbosity: 3,
                    quiet: false,
                },
                debug: true,
                logging: false,
            },
            Args::try_parse_from(["test", "-d", "report", "-p", "pat001", "-vvv"]).unwrap()
        );
    }

    #[test]
    fn test_run_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Run {
                    dry_run: false,
                    pattern: "pat001".to_string(),
                    parallel: false,
                    verbosity: 0,
                    quiet: false,
                },
                debug: false,
                logging: false,
            },
            Args::try_parse_from(["test", "run", "-p", "pat001"]).unwrap()
        );
    }

    #[test]
    fn test_show_defaults() {
        let args = Args::try_parse_from(["test", "show"]).unwrap();
        match args.command {
            Subcommands::Show { pattern, verbosity } => {
                assert_eq!(pattern, ".tdb");
                assert_eq!(verbosity, 0);
            }
            _ => panic!("expected Show"),
        }
    }

    #[test]
    fn test_show_with_verbosity() {
        let args = Args::try_parse_from(["test", "show", "-p", "hello", "-vv"]).unwrap();
        match args.command {
            Subcommands::Show { pattern, verbosity } => {
                assert_eq!(pattern, "hello");
                assert_eq!(verbosity, 2);
            }
            _ => panic!("expected Show"),
        }
    }

    #[test]
    fn test_show_alias() {
        let args = Args::try_parse_from(["test", "w", "-p", "foo"]).unwrap();
        match args.command {
            Subcommands::Show { pattern, .. } => assert_eq!(pattern, "foo"),
            _ => panic!("expected Show"),
        }
    }

    #[test]
    fn test_list_defaults() {
        let args = Args::try_parse_from(["test", "list"]).unwrap();
        match args.command {
            Subcommands::List { pattern } => assert_eq!(pattern, ".tdb"),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn test_list_with_pattern() {
        let args = Args::try_parse_from(["test", "list", "-p", "hello"]).unwrap();
        match args.command {
            Subcommands::List { pattern } => assert_eq!(pattern, "hello"),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn test_list_alias() {
        let args = Args::try_parse_from(["test", "l", "-p", "foo"]).unwrap();
        match args.command {
            Subcommands::List { pattern } => assert_eq!(pattern, "foo"),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn test_migrate_defaults() {
        let args = Args::try_parse_from(["test", "migrate"]).unwrap();
        match args.command {
            Subcommands::Migrate { pattern } => assert_eq!(pattern, ".tdb"),
            _ => panic!("expected Migrate"),
        }
    }

    #[test]
    fn test_migrate_with_pattern() {
        let args = Args::try_parse_from(["test", "migrate", "-p", "hello"]).unwrap();
        match args.command {
            Subcommands::Migrate { pattern } => assert_eq!(pattern, "hello"),
            _ => panic!("expected Migrate"),
        }
    }

    #[test]
    fn test_migrate_alias() {
        let args = Args::try_parse_from(["test", "m", "-p", "foo"]).unwrap();
        match args.command {
            Subcommands::Migrate { pattern } => assert_eq!(pattern, "foo"),
            _ => panic!("expected Migrate"),
        }
    }

    #[test]
    fn test_rebase_defaults() {
        let args = Args::try_parse_from(["test", "rebase"]).unwrap();
        match args.command {
            Subcommands::Rebase { pattern } => assert_eq!(pattern, ".tdb"),
            _ => panic!("expected Rebase"),
        }
    }

    #[test]
    fn test_rebase_with_pattern() {
        let args = Args::try_parse_from(["test", "rebase", "-p", "hello"]).unwrap();
        match args.command {
            Subcommands::Rebase { pattern } => assert_eq!(pattern, "hello"),
            _ => panic!("expected Rebase"),
        }
    }

    #[test]
    fn test_rebase_alias() {
        let args = Args::try_parse_from(["test", "u", "-p", "foo"]).unwrap();
        match args.command {
            Subcommands::Rebase { pattern } => assert_eq!(pattern, "foo"),
            _ => panic!("expected Rebase"),
        }
    }

    #[test]
    fn test_reset_defaults() {
        let args = Args::try_parse_from(["test", "reset"]).unwrap();
        match args.command {
            Subcommands::Reset { pattern } => assert_eq!(pattern, ".tdb"),
            _ => panic!("expected Reset"),
        }
    }

    #[test]
    fn test_reset_with_pattern() {
        let args = Args::try_parse_from(["test", "reset", "-p", "hello"]).unwrap();
        match args.command {
            Subcommands::Reset { pattern } => assert_eq!(pattern, "hello"),
            _ => panic!("expected Reset"),
        }
    }

    #[test]
    fn test_complete_defaults() {
        let args = Args::try_parse_from(["test", "complete"]).unwrap();
        match args.command {
            Subcommands::Complete { pattern } => assert_eq!(pattern, ".rgt"),
            _ => panic!("expected Complete"),
        }
    }

    #[test]
    fn test_complete_with_pattern() {
        let args = Args::try_parse_from(["test", "complete", "-p", "hello"]).unwrap();
        match args.command {
            Subcommands::Complete { pattern } => assert_eq!(pattern, "hello"),
            _ => panic!("expected Complete"),
        }
    }

    #[test]
    fn test_run_default_pattern() {
        let args = Args::try_parse_from(["test", "run"]).unwrap();
        match args.command {
            Subcommands::Run { pattern, .. } => assert_eq!(pattern, ".tdb"),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn test_report_default_pattern() {
        let args = Args::try_parse_from(["test", "report"]).unwrap();
        match args.command {
            Subcommands::Report { pattern, .. } => assert_eq!(pattern, ".tdb"),
            _ => panic!("expected Report"),
        }
    }

    #[test]
    fn test_run_no_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Run {
                    dry_run: true,
                    pattern: "pat001".to_string(),
                    parallel: false,
                    verbosity: 0,
                    quiet: false,
                },
                debug: true,
                logging: false,
            },
            Args::try_parse_from(["test", "-d", "run", "-p", "pat001", "-n"]).unwrap()
        );
    }
}
