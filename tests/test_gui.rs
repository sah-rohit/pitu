use pitu::cli::{Cli, Commands};
use clap::Parser;

#[test]
fn test_cli_gui_flag_parsing() {
    let args = Cli::parse_from(["pitu", "--gui"]);
    assert!(args.gui);
}

#[test]
fn test_cli_gui_subcommand_parsing() {
    let args = Cli::parse_from(["pitu", "gui"]);
    assert!(matches!(args.command, Some(Commands::Gui)));
}
