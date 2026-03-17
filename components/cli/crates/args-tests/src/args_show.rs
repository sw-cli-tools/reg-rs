use clap::Parser;
use reg_rs_args::args::Args;
use reg_rs_args::subcommands::Subcommands;

#[test]
fn test_list_with_pattern() {
    let args = <Args as Parser>::try_parse_from(["test", "list", "-p", "hello"])
        .expect("failed to parse CLI args");
    match args.command {
        Subcommands::List { pattern } => assert_eq!(pattern, "hello"),
        _ => panic!("expected List"),
    }
}

#[test]
fn test_list_alias() {
    let args = <Args as Parser>::try_parse_from(["test", "l", "-p", "foo"])
        .expect("failed to parse CLI args");
    match args.command {
        Subcommands::List { pattern } => assert_eq!(pattern, "foo"),
        _ => panic!("expected List"),
    }
}

#[test]
fn test_migrate_defaults() {
    let args =
        <Args as Parser>::try_parse_from(["test", "migrate"]).expect("failed to parse CLI args");
    match args.command {
        Subcommands::Migrate { pattern } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected Migrate"),
    }
}

#[test]
fn test_migrate_with_pattern() {
    let args = <Args as Parser>::try_parse_from(["test", "migrate", "-p", "hello"])
        .expect("failed to parse CLI args");
    match args.command {
        Subcommands::Migrate { pattern } => assert_eq!(pattern, "hello"),
        _ => panic!("expected Migrate"),
    }
}

#[test]
fn test_migrate_alias() {
    let args = <Args as Parser>::try_parse_from(["test", "m", "-p", "foo"])
        .expect("failed to parse CLI args");
    match args.command {
        Subcommands::Migrate { pattern } => assert_eq!(pattern, "foo"),
        _ => panic!("expected Migrate"),
    }
}

#[test]
fn test_rebase_defaults() {
    let args =
        <Args as Parser>::try_parse_from(["test", "rebase"]).expect("failed to parse CLI args");
    match args.command {
        Subcommands::Rebase { pattern } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected Rebase"),
    }
}
