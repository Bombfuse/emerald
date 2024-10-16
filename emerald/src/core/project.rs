use std::collections::{HashMap, HashSet};

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
}
