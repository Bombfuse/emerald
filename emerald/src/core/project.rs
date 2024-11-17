pub mod project_module;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::{HashMap, HashSet};
use project_module::ProjectModule;
use serde::{Deserialize, Serialize};

use crate::schedule::Schedule;

/// Metadata describing how to boot up the project, and handle it during runtime.
#[derive(Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub init_world: String,
    pub init_schedule: String,
    pub schedules: HashMap<String, Schedule>,
    pub version: String,
    pub systems: HashSet<String>,
    pub root_module: ProjectModule,
}
impl Default for Project {
    fn default() -> Self {
        Self {
            name: Default::default(),
            init_world: Default::default(),
            init_schedule: Default::default(),
            schedules: Default::default(),
            version: Default::default(),
            systems: Default::default(),
            root_module: ProjectModule::root(),
        }
    }
}

impl Project {
    pub fn add_system(&mut self, module_path_from_base_module: &str) {
        // parse out directory and system name from format of `path/to/module/system_name`
        let system_name = get_name_from_module_path(module_path_from_base_module).unwrap();
        // add system to the correct module

        // let system_name = last_word_in_path();
        // let module = get_module_from_path();
        // module.add_system(system);
    }

    pub fn add_module(&mut self, path: &str) {
        // parse out directory and module name from format of `path/to/module/module_name`
        // add new submodule to the correct module
    }

    pub fn add_component(&mut self, path: &str) {
        // parse out directory and component name from format of `path/to/module/component_name`
        // add component to the correct module
    }

    pub fn add_event(&mut self, path: &str) {
        // parse out directory and event name from format of `path/to/module/event_name`
        // add event to the correct module
    }
}

fn get_name_from_module_path(module_path: &str) -> Option<String> {
    let mut names = module_path
        .split("/")
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    if names.len() == 1 {
        return Some(names.remove(0));
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::project::get_name_from_module_path;

    #[test]
    fn get_name_from_module_path_succeeds_on_only_name() {
        let module_path = "test_module";
        let name = get_name_from_module_path(module_path).unwrap();
        assert_eq!(name, "test_module");
    }

    #[test]
    fn get_name_from_module_path_succeeds_on_multi_nested_module() {
        let module_path = "physics/movement/velocity";
        let name = get_name_from_module_path(module_path).unwrap();
        assert_eq!(name, "velocity");
    }
}
