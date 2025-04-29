use std::env;

use structopt::StructOpt;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

/// Argument processing configuration
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(
    name = "rtt1",
version = generated_version())]
pub struct Args {
    #[structopt(long, short)]
    /// Prints debugging info.  -d must preceed subcommands
    pub debug: bool,
    #[structopt(long, short)]
    /// Logs to a log file.  -l must preceed subcommands
    pub logging: bool,
    /// Subcommands
    #[structopt(subcommand)]
    pub command: Subcommands,
}

/// Regression Test Tool (first draft) - create and manage tests
/// - for more details: rtt1 &lt;subcommand&gt; -h
#[derive(Debug, PartialEq, StructOpt)]
pub enum Subcommands {
    /// Creates a new test of a specified command (alias c)
    #[structopt(name = "create", alias = "c")]
    Create {
        #[structopt(long, short)]
        /// Names the test (a database file to be created)
        test: String,
        #[structopt(long, short)]
        /// Specifies a command to be executed
        command: String,
    },
    /// Removes previously created test and run results if any.  Discards test and results!
    Remove {
        #[structopt(long, short)]
        /// Removes tests and results matching this naming pattern.  
        pattern: String,
    },
    /// Reports counts/summary of specified test(s) (alias p)
    #[structopt(name = "report", alias = "p")]
    Report {
        #[structopt(long, short)]
        /// name pattern to report on.  Can match zero, one, or more tests.
        pattern: String,
        /// Verbosity: -v adds names. -vv adds failure info. -vvv adds differences info.
        #[structopt(short, parse(from_occurrences))]
        verbosity: u8,
    },
    /// Runs a test (or tests) based on a test name pattern (alias r)
    #[structopt(name = "run", alias = "r")]
    Run {
        #[structopt(long, short)]
        /// Discovers tests matching this naming pattern
        pattern: String,
        /// Prints steps instead of executing them
        #[structopt(long, short = "n")]
        dry_run: bool,
    },
    /// Starts a server to monitor long running tests and/or show results (alias s)
    #[structopt(name = "status", alias = "s")]
    Status {
        #[structopt(long, short)]
        /// Monitors tests matching this naming pattern
        pattern: String,
        #[structopt(default_value = "4111", long, short)]
        /// optional port number
        localhost_port: u16,
    },
}
/// Parse arguments
pub fn parse_args() -> Args {
    Args::from_args()
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
            Args::from_iter(&["test", "create", "-t", "pat001", "-c", "pwd"])
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
            Args::from_iter(&["test", "-d", "create", "-t", "pat001", "-c", "pwd"])
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
            Args::from_iter(&["test", "remove", "-p", "pat001"])
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
            Args::from_iter(&["test", "-d", "remove", "-p", "pat001"])
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
            Args::from_iter(&["test", "report", "-p", "pat001"])
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
            Args::from_iter(&["test", "-d", "report", "-p", "pat001", "-vvv"])
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
            Args::from_iter(&["test", "run", "-p", "pat001"])
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
            Args::from_iter(&["test", "-d", "run", "-p", "pat001", "-n"])
        );
    }
}
