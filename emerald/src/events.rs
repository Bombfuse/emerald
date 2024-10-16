use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use anymap::{any::Any, AnyMap};

use crate::{Emerald, World};
pub type EventHandler<T> = fn(emd: &mut Emerald, world: &mut World, event: T);

/// Handles
pub struct Events {
    events: anymap::Map<dyn Any + Send + Sync + 'static>,
    total_count: usize,
    type_ids: HashSet<TypeId>,
}
impl Events {
    pub fn new() -> Self {
        Self {
            events: anymap::Map::new(),
            total_count: 0,
            type_ids: HashSet::new(),
        }
    }
    pub fn get_events<T: 'static + Send + Sync>(&self) -> Option<&Vec<T>> {
        self.events.get::<Vec<T>>()
    }

    pub fn push<T: 'static + Send + Sync>(&mut self, event: T) {
        if !self.events.contains::<Vec<T>>() {
            self.events.insert(Vec::<T>::new());
        }

        self.events
            .get_mut::<Vec<T>>()
            .map(|events| {
                events.push(event);
                1
            })
            .map(|amount| self.total_count += amount);
    }

    /// Pops the first event off of a list of events of this type
    pub fn pop<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        self.events
            .get_mut::<Vec<T>>()
            .map(|events| events.pop())
            .flatten()
            .map(|event| {
                self.total_count -= 1;
                event
            })
    }

    pub fn count<T: 'static + Send + Sync>(&mut self) -> usize {
        self.events
            .get::<Vec<T>>()
            .map(|events| events.len())
            .unwrap_or(0)
    }

    pub fn total_count(&self) -> usize {
        self.total_count
    }
}

pub fn get_events_count(world: &World) -> usize {
    world
        .resources_ref()
        .get::<Events>()
        .map(|e| e.total_count())
        .unwrap_or(0)
}

fn insert_events_resource_if_not_exists(world: &mut World) {
    if !world.resources().contains::<Events>() {
        world.resources().insert(Events::new());
    }
}

pub fn get_events<T: 'static + Send + Sync>(world: &World) -> Option<&Vec<T>> {
    world
        .resources_ref()
        .get::<Events>()
        .map(|events| events.get_events::<T>())
        .flatten()
}

pub fn get_events_len<T: 'static + Send + Sync>(world: &mut World) -> usize {
    world
        .resources()
        .get_mut::<Events>()
        .map(|e| e.count::<T>())
        .unwrap_or(0)
}

pub fn push_events<T: 'static + Send + Sync>(world: &mut World, events: Vec<T>) {
    for event in events {
        push_event(world, event);
    }
}

pub fn get_event_types(world: &World) -> HashSet<TypeId> {
    world
        .resources_ref()
        .get::<Events>()
        .map(|e| e.type_ids.clone())
        .unwrap_or_default()
}

pub fn push_event<T: 'static + Send + Sync>(world: &mut World, event: T) {
    insert_events_resource_if_not_exists(world);
    let type_id = TypeId::of::<T>();

    world.resources().get_mut::<Events>().map(|e| {
        e.push(event);
        e.type_ids.insert(type_id);
    });
}
pub fn pop_event<T: 'static + Send + Sync>(world: &mut World) -> Option<T> {
    world
        .resources()
        .get_mut::<Events>()
        .map(|e| e.pop::<T>())
        .flatten()
}

pub fn pop_events<T: 'static + Send + Sync>(world: &mut World) -> Vec<T> {
    let mut events = Vec::new();

    while let Some(e) = pop_event::<T>(world) {
        events.push(e);
    }

    events
}

/// An event of the given type is either inserted or pushed into the events queue/world
pub fn event_exists<'a, T: 'static + Send + Sync>(world: &'a mut World) -> bool {
    get_events_len::<T>(world) > 0
}

/// Consumes all of the events in the queue and handles them
pub fn consume_event_queue(emd: &mut Emerald, world: &mut World) {
    if let Some(event_handlers) = emd.resources().remove::<EventHandlerRegistry>() {
        // Handle events until event queue is completely handled
        while get_events_count(&world) > 0 {
            for event_type_id in get_event_types(&world) {
                if !event_handlers.contains_key(&event_type_id) {
                    panic!(
                        "Event pushed without a registered handler {:?}",
                        event_type_id
                    );
                }
            }
            for (_, handler) in &event_handlers {
                (handler)(emd, world);
            }
        }
        emd.resources().insert(event_handlers);
    }
}

pub type EventHandlerRegistry =
    HashMap<TypeId, Box<dyn Fn(&mut Emerald, &mut World) + Send + Sync>>;

pub fn add_event_handler<E: Any + 'static + Send + Sync>(
    emd: &mut Emerald,
    handler: EventHandler<E>,
) {
    if !emd.resources().contains::<EventHandlerRegistry>() {
        emd.resources().insert(EventHandlerRegistry::new());
    }

    emd.resources()
        .get_mut::<EventHandlerRegistry>()
        .map(|registry| {
            let type_id = TypeId::of::<E>();
            registry.insert(
                type_id,
                Box::new(move |emd, world| {
                    while let Some(event) = world
                        .resources()
                        .get_mut::<Events>()
                        .map(|events| events.pop::<E>())
                        .flatten()
                    {
                        (handler)(emd, world, event);
                    }
                }),
            )
        });
}

#[cfg(test)]
mod tests {

    use crate::{events::get_events_len, World};

    use super::{push_event, Events};

    struct TestEvent {}

    #[test]
    fn add_and_remove_one_event_from_list() {
        let mut events = Events::new();
        assert_eq!(events.total_count(), 0);
        events.push(TestEvent {});
        assert_eq!(events.total_count(), 1);
        assert!(events.pop::<TestEvent>().is_some());
        assert_eq!(events.total_count(), 0);
    }

    #[test]
    fn get_events_len_test() {
        let mut world = World::new();
        push_event(&mut world, TestEvent {});
        assert!(get_events_len::<TestEvent>(&mut world) == 1);
    }
}
