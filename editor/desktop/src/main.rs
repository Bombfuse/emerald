use std::collections::{HashMap, HashSet};

use emerald::Schedule;

fn main() {
    let mut schedules = HashMap::new();
    schedules.insert("default".to_string(), Schedule::new());

    emd_desktop::start(
        emerald::Project {
            init_world: "".to_string(),
            init_schedule: "default".to_string(),
            schedules,
            name: "".to_string(),
            version: "".to_string(),
            systems: HashSet::new(),
        },
        Default::default(),
    );
}
