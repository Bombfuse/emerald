use std::ffi::OsString;

use clap::{arg, Command, Parser};
use commands::{
    generate::{cli_generate, subcommand_generate},
    new::{cli_new, subcommand_new},
    run::subcommand_run,
};
pub mod commands;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,
    // /// Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
}

fn cli() -> Command {
    Command::new("emd")
        .about("A CLI for creating, editing, managing, and running Emerald projects")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(subcommand_new())
        .subcommand(subcommand_run())
        .subcommand(subcommand_generate())
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("new", submatches)) => {
            cli_new(submatches);
        }
        Some(("generate", submatches)) => {
            cli_generate(submatches);
        }
        Some(("g", submatches)) => {
            cli_generate(submatches);
        }
        Some((ext, sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            println!("Calling out to {ext:?} with {args:?}");
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn generate_new_project() {
        let command = "emd new test_project";
        assert!(todo!("test_project was generated"));
    }
}
