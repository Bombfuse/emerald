use std::collections::HashMap;

use crate::{Emerald, World};

pub type System = fn(emd: &mut Emerald, world: &mut World);

#[derive(Default)]
struct SystemRegistry {
    pub systems: HashMap<String, System>,
}

fn init_system_registry_if_not_initted(emd: &mut Emerald) {
    if !emd.resources().contains::<SystemRegistry>() {
        emd.resources().insert(SystemRegistry::default());
    }
}

pub fn register_system(emd: &mut Emerald, name: String, system: System) {
    init_system_registry_if_not_initted(emd);

    emd.resources().get_mut::<SystemRegistry>().map(|registry| {
        registry.systems.insert(name, system);
    });
}

pub fn unregister_system(emd: &mut Emerald, name: String) -> Option<System> {
    emd.resources()
        .get_mut::<SystemRegistry>()
        .map(|registry| registry.systems.remove(&name))
        .flatten()
}

pub fn get_system(emd: &mut Emerald, name: &str) -> Option<System> {
    emd.resources()
        .get::<SystemRegistry>()
        .map(|registry| registry.systems.get(name))
        .flatten()
        .map(|system| system.clone())
}
