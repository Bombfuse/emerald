use alloc::string::{String, ToString};
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct ProjectModule {
    /// Names of this module's sub modules
    pub sub_modules: HashMap<String, ProjectModule>,
    /// Names of this module's systems
    pub systems: HashSet<String>,
    /// Names of this module's components
    pub components: HashSet<String>,
    /// Names of this module's events
    pub events: HashSet<String>,

    pub name: String,
}
impl ProjectModule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn root() -> Self {
        Self {
            name: "root".to_string(),
            ..Default::default()
        }
    }
}
