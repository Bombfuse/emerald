use std::{
    env::current_dir,
    fs::{self, create_dir, File},
    io::Write,
};

use crate::generate_system::generate_system_file;

pub fn generate_module(name: &str) {
    let current_module = get_current_module_file_name();
    println!("current module {:?}", current_module);
    create_dir(name).unwrap();
    let file = File::create(format!("{}.rs", name)).unwrap();
    file.write_all(generate_module_file_content_from_template(name));

    regenerate_parent_module();
}

fn generate_module_file_content_from_template(name: &str) -> String {
    "".to_string()
}

fn get_current_module_file_name() -> String {
    let is_top_module = fs::read_dir("./").into_iter().any(|mut read_dir| {
        read_dir.any(|file| {
            file.ok()
                .map(|dir| dir.file_name().to_str().unwrap().contains("lib"))
                .is_some()
        })
    });
    let name = if is_top_module {
        "lib".to_string()
    } else {
        current_dir().unwrap().display().to_string()
    };

    name
}
