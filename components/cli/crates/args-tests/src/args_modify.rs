use clap::Parser;
use reg_rs_args::args::Args;
use reg_rs_args::subcommands::Subcommands;

#[test]
fn test_rebase_with_pattern() {
    let args = <Args as Parser>::try_parse_from(["test", "rebase", "-p", "hello"]).unwrap();
    match args.command {
        Subcommands::Rebase { pattern } => assert_eq!(pattern, "hello"),
        _ => panic!("expected Rebase"),
    }
}

#[test]
fn test_rebase_alias() {
    let args = <Args as Parser>::try_parse_from(["test", "u", "-p", "foo"]).unwrap();
    match args.command {
        Subcommands::Rebase { pattern } => assert_eq!(pattern, "foo"),
        _ => panic!("expected Rebase"),
    }
}

#[test]
fn test_reset_defaults() {
    let args = <Args as Parser>::try_parse_from(["test", "reset"]).unwrap();
    match args.command {
        Subcommands::Reset { pattern } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected Reset"),
    }
}

#[test]
fn test_reset_with_pattern() {
    let args = <Args as Parser>::try_parse_from(["test", "reset", "-p", "hello"]).unwrap();
    match args.command {
        Subcommands::Reset { pattern } => assert_eq!(pattern, "hello"),
        _ => panic!("expected Reset"),
    }
}

#[test]
fn test_complete_defaults() {
    let args = <Args as Parser>::try_parse_from(["test", "complete"]).unwrap();
    match args.command {
        Subcommands::Complete { pattern } => assert_eq!(pattern, ".rgt"),
        _ => panic!("expected Complete"),
    }
}

#[test]
fn test_complete_with_pattern() {
    let args = <Args as Parser>::try_parse_from(["test", "complete", "-p", "hello"]).unwrap();
    match args.command {
        Subcommands::Complete { pattern } => assert_eq!(pattern, "hello"),
        _ => panic!("expected Complete"),
    }
}
