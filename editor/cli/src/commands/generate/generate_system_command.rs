use std::path::Path;

use clap::{arg, ArgMatches, Command};

pub fn subcommand_generate_system() -> Command {
    Command::new("system")
        .about("Generate a system for the current module")
        .arg(arg!(<SYSTEM_NAME> "The name of the system"))
        .arg_required_else_help(true)
}

pub fn cli_generate_system(submatches: &ArgMatches) {
    let system_name = submatches
        .get_one::<String>("SYSTEM_NAME")
        .expect("required");
    assert_eq!(Path::new(&system_name).exists(), false);
    generate_system(system_name);
}
