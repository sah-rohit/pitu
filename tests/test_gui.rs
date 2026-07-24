use pitu::cli::{Cli, Commands};
use clap::Parser;

#[test]
fn test_cli_rebase_parsing() {
    let args = Cli::parse_from(["pitu", "rebase", "photo.png"]);
    if let Some(Commands::Rebase { file }) = args.command {
        assert_eq!(file, "photo.png");
    } else {
        panic!("Failed to parse rebase subcommand");
    }
}

#[test]
fn test_cli_revert_parsing() {
    let args = Cli::parse_from(["pitu", "revert", "photo.png", "abc1234"]);
    if let Some(Commands::Revert { file, commit }) = args.command {
        assert_eq!(file, "photo.png");
        assert_eq!(commit, "abc1234");
    } else {
        panic!("Failed to parse revert subcommand");
    }
}
