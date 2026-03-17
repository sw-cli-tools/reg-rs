use reg_rs_args::subcommands::Subcommands;

/// Options for the create subcommand
pub struct CreateOptions {
    /// Test name
    pub test: String,
    /// Shell command to execute
    pub command: Option<String>,
    /// AI-generated command description
    pub describe: Option<String>,
    /// Context command for AI generation
    pub context: Option<String>,
    /// Preprocessing command
    pub preprocess: Option<String>,
    /// Diff normalization mode
    pub diff_mode: String,
    /// Command timeout in seconds
    pub timeout: u64,
    /// Test description
    pub desc: Option<String>,
    /// Expected behavior
    pub expects: Option<String>,
    /// Flakiness notes
    pub flaky_note: Option<String>,
}

/// Configuration data
#[derive(Debug)]
pub struct Config {
    /// subcommands
    pub mode: Subcommands,
    /// debugging flag
    pub debug: bool,
}

impl Config {
    /// Determines if dry-run flag was specified
    pub fn is_dry_run(&self) -> bool {
        match &self.mode {
            Subcommands::Run { dry_run, .. } => *dry_run,
            _ => false,
        }
    }

    /// Determines if parallel flag was specified
    pub fn is_parallel(&self) -> bool {
        match &self.mode {
            Subcommands::Run { parallel, .. } => *parallel,
            _ => false,
        }
    }

    /// Extracts the test name pattern
    pub fn extract_pattern(&self) -> &str {
        log::debug!("extract_pattern mode: {:?}", &self.mode);
        let p = match &self.mode {
            Subcommands::Create { .. } => unreachable!(),
            Subcommands::Analyze { pattern, .. } => pattern,
            Subcommands::Complete { pattern, .. } => pattern,
            Subcommands::List { pattern, .. } => pattern,
            Subcommands::Migrate { pattern, .. } => pattern,
            Subcommands::Rebase { pattern, .. } => pattern,
            Subcommands::Reset { pattern, .. } => pattern,
            Subcommands::Show { pattern, .. } => pattern,
            Subcommands::Remove { pattern, .. } => pattern,
            Subcommands::Report { pattern, .. } => pattern,
            Subcommands::Run { pattern, .. } => pattern,
            Subcommands::Status { pattern, .. } => pattern,
        };
        log::debug!("extract_pattern result: {:?}", &p);
        p
    }

    /// Extracts all create subcommand options as a single struct
    pub fn extract_create_options(&self) -> CreateOptions {
        if let Subcommands::Create {
            test,
            command,
            describe,
            context,
            preprocess,
            diff_mode,
            timeout,
            desc,
            expects,
            flaky_note,
        } = &self.mode
        {
            CreateOptions {
                test: test.clone(),
                command: command.clone(),
                describe: describe.clone(),
                context: context.clone(),
                preprocess: preprocess.clone(),
                diff_mode: diff_mode.clone(),
                timeout: *timeout,
                desc: desc.clone(),
                expects: expects.clone(),
                flaky_note: flaky_note.clone(),
            }
        } else {
            panic!("extract_create_options called on non-Create config")
        }
    }

    /// Determines how verbose the output should be
    pub fn verbosity_level(&self) -> u8 {
        match &self.mode {
            Subcommands::Report { verbosity, .. }
            | Subcommands::Run { verbosity, .. }
            | Subcommands::Show { verbosity, .. } => {
                log::debug!("verbosity_level: {}", verbosity);
                *verbosity
            }
            _ => 0,
        }
    }

    /// Determines if quiet mode was specified
    pub fn is_quiet(&self) -> bool {
        match &self.mode {
            Subcommands::Report { quiet, .. } | Subcommands::Run { quiet, .. } => *quiet,
            _ => false,
        }
    }

    /// Determines which port to use for web server
    pub fn status_port(&self) -> u16 {
        if let Subcommands::Status { localhost_port, .. } = &self.mode {
            *localhost_port
        } else {
            reg_rs_types::constants::DEFAULT_STATUS_PORT
        }
    }
}
