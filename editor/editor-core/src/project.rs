use std::{
    fs::{create_dir, File},
    io::Write,
};

use emerald::Project;

pub fn generate_project(project: Project) {
    create_dir(&project.name).unwrap();
    let _workspace_file = File::create(format!("{}/Cargo.toml", &project.name)).unwrap();
    let project_name = project.name.clone();
    let mut project_file =
        File::create(format!("{}/{}.emd", &project.name, &project.name)).unwrap();
    project_file
        .write_all(project_to_file_content(project).as_bytes())
        .unwrap();

    //  cargo new --lib project_name
    std::process::Command::new("cargo")
        .args(["new", "--lib", &project_name])
        .current_dir(format!("./{}", &project_name))
        .output()
        .unwrap();
}

fn project_to_file_content(project: Project) -> String {
    "".into()
}
