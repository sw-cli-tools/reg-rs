use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about="using structopt to define subcommands create, report, and run")]
pub enum Subcommands {
    /// create a new test of a specified command
    Create {
        #[structopt(long, short)]
        test: String,
        #[structopt(long, short)]
        command: String,
    },
    /// report results of previously run test(s)
    Report{
    },
    /// run a test (or tests) based on a test name pattern
    Run{
        #[structopt(long, short)]
        pattern: String,
        #[structopt(long, short="n")]
        dry_run: bool,
    },
}
pub fn subcommands() -> Subcommands {
    Subcommands::from_args()
}
