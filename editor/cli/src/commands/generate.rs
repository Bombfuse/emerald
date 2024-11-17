pub mod generate_component_command;
pub mod generate_event_command;
pub mod generate_module_command;
pub mod generate_system_command;

use clap::{ArgMatches, Command};
use generate_component_command::{cli_generate_component, subcommand_generate_component};
use generate_event_command::{cli_generate_event, subcommand_generate_event};
use generate_module_command::{cli_generate_module, subcommand_generate_module};
use generate_system_command::{cli_generate_system, subcommand_generate_system};

pub fn subcommand_generate() -> Command {
    Command::new("generate")
        .subcommand(subcommand_generate_system())
        .subcommand(subcommand_generate_event())
        .subcommand(subcommand_generate_component())
        .subcommand(subcommand_generate_module())
}

pub fn cli_generate(submatches: &ArgMatches) {
    match submatches.subcommand() {
        Some(("module", submatches)) => {
            cli_generate_module(submatches);
        }
        Some(("component", submatches)) => {
            cli_generate_component(submatches);
        }
        Some(("system", submatches)) => {
            cli_generate_system(submatches);
        }
        Some(("event", submatches)) => {
            cli_generate_event(submatches);
        }
        _ => {}
    }
}
