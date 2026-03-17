use clap::Parser;
use reg_rs_args::args::Args;
use reg_rs_args::subcommands::Subcommands;

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
        <Args as Parser>::try_parse_from(["test", "create", "-t", "pat001", "-c", "pwd"])
            .expect("failed to parse CLI args")
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
        <Args as Parser>::try_parse_from(["test", "-d", "create", "-t", "pat001", "-c", "pwd"])
            .expect("failed to parse CLI args")
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
        <Args as Parser>::try_parse_from(["test", "remove", "-p", "pat001"])
            .expect("failed to parse CLI args")
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
        <Args as Parser>::try_parse_from(["test", "-d", "remove", "-p", "pat001"])
            .expect("failed to parse CLI args")
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
        <Args as Parser>::try_parse_from(["test", "report", "-p", "pat001"])
            .expect("failed to parse CLI args")
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
        <Args as Parser>::try_parse_from(["test", "-d", "report", "-p", "pat001", "-vvv"])
            .expect("failed to parse CLI args")
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
        <Args as Parser>::try_parse_from(["test", "run", "-p", "pat001"])
            .expect("failed to parse CLI args")
    );
}
