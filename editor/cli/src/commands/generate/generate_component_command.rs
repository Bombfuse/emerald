use std::path::Path;

use clap::{arg, ArgMatches, Command};

pub fn subcommand_generate_component() -> Command {
    Command::new("component")
        .about("Generate a component with the given name")
        .arg(arg!(<COMPONENT_NAME> "The name of the component"))
        .arg_required_else_help(true)
}
pub fn cli_generate_component(submatches: &ArgMatches) {
    let component_name = submatches
        .get_one::<String>("COMPONENT_NAME")
        .expect("required");
    assert_eq!(Path::new(&component_name).exists(), false);
    generate_component(component_name);
}
