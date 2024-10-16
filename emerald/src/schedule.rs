use hecs::World;
use serde::{Deserialize, Serialize};

use crate::{
    events::{consume_event_queue, get_event_types, get_events_count, EventHandlerRegistry},
    system::{get_system, System},
    world_stack::WorldStack,
    Emerald,
};

#[derive(Debug, PartialEq, Eq)]
enum ScheduleEvent {
    Cancel,
    SwitchWorld { world: String },
    PopWorld,
    PushWorld { world: String },
    SwitchSchedule { new_schedule: Schedule },
}

fn switch_schedule(emd: &mut Emerald, new_schedule: Schedule) {
    emd.resources().remove::<Schedule>();
    emd.resources().insert(new_schedule);
}

/// A schedule determines what systems to run, and in what order, from the `SystemRegistry`
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Schedule {
    pub systems: Vec<String>,
}
impl Schedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn run(&mut self, emd: &mut Emerald, world_stack: &mut WorldStack) {
        let systems = self
            .systems
            .iter()
            .map(|name| get_system(emd, name).unwrap())
            .collect::<Vec<System>>();

        let world = world_stack.pop_front();
        if let Some(mut world) = world {
            for system in systems {
                (system)(emd, &mut world);

                consume_event_queue(emd, &mut world);

                while let Some(event) = world.resources().remove::<ScheduleEvent>() {
                    match event {
                        ScheduleEvent::Cancel => {
                            return;
                        }
                        ScheduleEvent::SwitchWorld { world: new_world } => {
                            let new_world = emd.loader().world(new_world).unwrap();
                            world_stack.push_front(new_world);
                            return;
                        }
                        ScheduleEvent::PopWorld => {
                            return;
                        }
                        ScheduleEvent::PushWorld { world: new_world } => {
                            let new_world = emd.loader().world(new_world).unwrap();
                            world_stack.push_front(new_world);
                        }
                        ScheduleEvent::SwitchSchedule { new_schedule } => {
                            switch_schedule(emd, new_schedule);
                            return;
                        }
                    }
                }
            }
            world_stack.push_front(world);
        }
    }
}
