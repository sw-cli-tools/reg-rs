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

WORKFLOW:
  1. Create a test:   reg-rs create -t my_test.tdb -c 'my_command'
  2. Run the test:    reg-rs run -p my_test.tdb
  3. View results:    reg-rs report -p my_test.tdb -v

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

EXAMPLES:
  reg-rs create -t data/pwd_test.tdb -c 'pwd'
  reg-rs create -t data/version.tdb -c 'git --version'
  reg-rs c-t tests/ls.tdb -c 'ls -la'  # using alias"
    )]
    Create {
        /// Path to the test database file to create (e.g., data/my_test.tdb)
        #[clap(long, short)]
        test: String,
        /// Shell command to execute and capture (e.g., 'echo hello')
        #[clap(long, short)]
        command: String,
    },
    /// Removes previously created test and run results if any.  Discards test and results!
    #[clap(
        long_about = "Removes test database files matching the specified pattern.

WARNING: This permanently deletes the test and all stored results!

EXAMPLES:
  reg-rs remove -p data/old_test.tdb
  reg-rs remove -p 'data/temp_*.tdb'"
    )]
    Remove {
        /// Glob pattern to match test files to remove (e.g., 'data/*.tdb')
        #[clap(long, short)]
        pattern: String,
    },
    /// Reports counts/summary of specified test(s) (alias p)
    #[clap(
        name = "report",
        alias = "p",
        long_about = "Reports on test results with configurable verbosity.

VERBOSITY LEVELS:
  (none)  - Show only summary counts
  -v      - Also show test names
  -vv     - Also show failure information
  -vvv    - Also show detailed differences

EXAMPLES:
  reg-rs report -p data/my_test.tdb         # basic summary
  reg-rs report -p 'data/*.tdb' -v          # show names
  reg-rs p-p data/my_test.tdb -vvv         # full details (using alias)"
    )]
    Report {
        /// Glob pattern to match test files (e.g., 'data/*.tdb')
        #[clap(long, short)]
        pattern: String,
        /// Verbosity: -v adds names, -vv adds failures, -vvv adds differences
        #[clap(short, action = clap::ArgAction::Count)]
        verbosity: u8,
    },
    /// Runs a test (or tests) based on a test name pattern (alias r)
    #[clap(
        name = "run",
        alias = "r",
        long_about = "Runs previously created tests and compares results against baselines.

Each matching test's command is re-executed, and the new output is compared
against the stored baseline. Any differences are recorded as potential regressions.

EXAMPLES:
  reg-rs run -p data/my_test.tdb           # run a specific test
  reg-rs run -p 'data/*.tdb'               # run all matching tests
  reg-rs r-p data/my_test.tdb -n          # dry-run (show what would run)"
    )]
    Run {
        /// Glob pattern to match test files to run (e.g., 'data/*.tdb')
        #[clap(long, short)]
        pattern: String,
        /// Show what would be run without actually executing
        #[clap(long, short = 'n')]
        dry_run: bool,
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
  reg-rs status -p 'data/*.tdb'            # start on default port 4111
  reg-rs status -p 'data/*.tdb' -l 8080    # use custom port
  reg-rs s-p 'data/*.tdb'                 # using alias"
    )]
    Status {
        /// Glob pattern to match test files to monitor (e.g., 'data/*.tdb')
        #[clap(long, short)]
        pattern: String,
        /// Port number for the web server (default: 4111)
        #[clap(default_value = "4111", long, short)]
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
                    command: "pwd".to_string(),
                },
                debug: false,
                logging: false,
            },
            Args::try_parse_from(&["test", "create", "-t", "pat001", "-c", "pwd"]).unwrap()
        );
    }

    #[test]
    fn test_create_no_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Create {
                    test: "pat001".to_string(),
                    command: "pwd".to_string(),
                },
                debug: true,
                logging: false,
            },
            Args::try_parse_from(&["test", "-d", "create", "-t", "pat001", "-c", "pwd"]).unwrap()
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
            Args::try_parse_from(&["test", "remove", "-p", "pat001"]).unwrap()
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
            Args::try_parse_from(&["test", "-d", "remove", "-p", "pat001"]).unwrap()
        );
    }

    #[test]
    fn test_report_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Report {
                    pattern: "pat001".to_string(),
                    verbosity: 0,
                },
                debug: false,
                logging: false,
            },
            Args::try_parse_from(&["test", "report", "-p", "pat001"]).unwrap()
        );
    }

    #[test]
    fn test_report_no_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Report {
                    pattern: "pat001".to_string(),
                    verbosity: 3,
                },
                debug: true,
                logging: false,
            },
            Args::try_parse_from(&["test", "-d", "report", "-p", "pat001", "-vvv"]).unwrap()
        );
    }

    #[test]
    fn test_run_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Run {
                    dry_run: false,
                    pattern: "pat001".to_string(),
                },
                debug: false,
                logging: false,
            },
            Args::try_parse_from(&["test", "run", "-p", "pat001"]).unwrap()
        );
    }

    #[test]
    fn test_run_no_defaults() {
        assert_eq!(
            Args {
                command: Subcommands::Run {
                    dry_run: true,
                    pattern: "pat001".to_string(),
                },
                debug: true,
                logging: false,
            },
            Args::try_parse_from(&["test", "-d", "run", "-p", "pat001", "-n"]).unwrap()
        );
    }
}
