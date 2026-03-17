use clap::Parser;
use reg_rs_args::args::Args;
use reg_rs_args::subcommands::Subcommands;

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
        <Args as Parser>::try_parse_from(["test", "-d", "run", "-p", "pat001", "-n"]).unwrap()
    );
}

#[test]
fn test_run_default_pattern() {
    let args = <Args as Parser>::try_parse_from(["test", "run"]).unwrap();
    match args.command {
        Subcommands::Run { pattern, .. } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected Run"),
    }
}

#[test]
fn test_report_default_pattern() {
    let args = <Args as Parser>::try_parse_from(["test", "report"]).unwrap();
    match args.command {
        Subcommands::Report { pattern, .. } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected Report"),
    }
}

#[test]
fn test_show_defaults() {
    let args = <Args as Parser>::try_parse_from(["test", "show"]).unwrap();
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
    let args = <Args as Parser>::try_parse_from(["test", "show", "-p", "hello", "-vv"]).unwrap();
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
    let args = <Args as Parser>::try_parse_from(["test", "w", "-p", "foo"]).unwrap();
    match args.command {
        Subcommands::Show { pattern, .. } => assert_eq!(pattern, "foo"),
        _ => panic!("expected Show"),
    }
}

#[test]
fn test_list_defaults() {
    let args = <Args as Parser>::try_parse_from(["test", "list"]).unwrap();
    match args.command {
        Subcommands::List { pattern } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected List"),
    }
}
