use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Regression Test Tool (first draft) - create, report, and run tests")]
pub struct Args {
    #[structopt(long, short)]
    /// Prints debugging info
    pub debug: bool,
    #[structopt(subcommand)]
    pub command: Subcommands,
}

#[derive(Debug, StructOpt)]
pub enum Subcommands {
    /// Creates a new test of a specified command
    Create {
        #[structopt(long, short)]
        /// Names the test (a database file to be created)
        test: String,
        #[structopt(long, short)]
        /// Specifies a command to be executed
        command: String,
    },
    /// Reports results of previously run test(s)
    Report {
        #[structopt(long, short)]
        /// Reports on tests mathing this naming pattern
        pattern: String,
    },
    /// Runs a test (or tests) based on a test name pattern
    Run {
        #[structopt(long, short)]
        /// Discovers tests matching this naming pattern
        pattern: String,
        /// Prints steps instead of executing them
        #[structopt(long, short = "n")]
        dry_run: bool,
    },
}
pub fn parse_args() -> Args {
    Args::from_args()
}
