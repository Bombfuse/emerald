use std::path::Path;

use clap::{arg, ArgMatches, Command};

pub fn subcommand_generate_event() -> Command {
    Command::new("event")
        .about("Generate an event and event handler with the given name")
        .arg(arg!(<EVENT_NAME> "The name of the event"))
        .arg_required_else_help(true)
}

pub fn cli_generate_event(submatches: &ArgMatches) {
    let event_name = submatches
        .get_one::<String>("EVENT_NAME")
        .expect("required");
    assert_eq!(Path::new(&event_name).exists(), false);
    generate_event(system_name);
}
