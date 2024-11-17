use std::path::Path;

use clap::{arg, ArgMatches, Command};
use editor_core::module::generate_module;

pub fn subcommand_generate_module() -> Command {
    Command::new("module")
        .about("Generate a submodule with the given name")
        .arg(arg!(<MODULE_NAME> "The name of the module"))
        .arg_required_else_help(true)
}

pub fn cli_generate_module(submatches: &ArgMatches) {
    let module_name = submatches
        .get_one::<String>("MODULE_NAME")
        .expect("required");
    assert_eq!(Path::new(&module_name).exists(), false);
    generate_module(module_name);
}
