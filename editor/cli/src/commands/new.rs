use std::path::Path;

use clap::{arg, ArgMatches, Command};
use editor_core::project::generate_project;
use emerald::Project;

pub fn cli_new(sub_matches: &ArgMatches) {
    let project_name = sub_matches
        .get_one::<String>("PROJECT_NAME")
        .expect("required");

    assert_eq!(Path::new(project_name).exists(), false);
    let mut project = Project::default();
    project.name = project_name.clone();
    generate_project(project);
}

pub fn subcommand_new() -> Command {
    Command::new("new")
        .about("Initializes a new Emerald project")
        .arg(arg!(<PROJECT_NAME> "The name of the project"))
        .arg_required_else_help(true)
}
