use std::{
    collections::HashSet,
    env::current_dir,
    fs::{self, create_dir, File},
    io::{Read, Write},
};

use emerald::{
    serde::{Deserialize, Serialize},
    serde_json,
};

#[derive(Deserialize, Serialize)]
#[serde(crate = "emerald::serde")]
pub struct EmeraldModule {
    /// Names of this module's submodules
    sub_modules: HashSet<String>,

    /// Name of this module
    name: String,

    /// Names of this module's components
    components: HashSet<String>,

    /// Names of this module's systems
    systems: HashSet<String>,

    /// Names of this module's events
    events: HashSet<String>,
}
impl EmeraldModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            sub_modules: Default::default(),
            components: Default::default(),
            systems: Default::default(),
            events: Default::default(),
        }
    }
}

pub fn generate_module(name: &str) {
    get_parent_module().map(|mut parent_module| {
        add_sub_module(&mut parent_module, name);
    });
}

fn get_parent_module_name() -> Option<String> {
    None
}

fn get_parent_module() -> Option<EmeraldModule> {
    get_parent_module_name().map(|parent_name| {
        let mut file = File::open(format!("../{}.emd_module", &parent_name)).unwrap();
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).unwrap();
        serde_json::from_str(&file_contents).unwrap()
    })
}

fn write_module(path: &str, module: &EmeraldModule) {
    let file_path = format!("{}.emd", path);
    let mut file = File::create(file_path).unwrap();
    file.write_all(serde_json::to_string(module).unwrap().as_bytes())
        .unwrap();
}

/// Adds a child module to the parent module then updates the parent module
fn add_sub_module(parent_module: &mut EmeraldModule, child_module: &str) {
    parent_module.sub_modules.insert(child_module.to_string());
    write_parent_module(parent_module);
    write_new_module(child_module);
}

fn write_parent_module(module: &EmeraldModule) {
    write_module("../", module);
}
fn write_new_module(name: &str) {
    write_module("./", &EmeraldModule::new(name.to_string()));
}

fn generate_module_rust_file_content_from_template(name: &str) -> String {
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
