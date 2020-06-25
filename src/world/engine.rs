use crate::world::*;
use crate::world::physics::*;

use legion::prelude::{Universe};
use legion::entity::*;
use legion::filter::*;
use legion::world::{TagSet, TagLayout, IntoComponentSource};

pub struct WorldEngine {
    world_stack: Vec<World>,
    universe: Universe,
}
impl WorldEngine {
    pub(crate) fn new() -> Self {
        let universe = Universe::new();

        WorldEngine {
            world_stack: Vec::new(),
            universe,
        }
    }


    /// Public API ///
    
    /// Create a new world and hand it to the user.
    pub fn create(&mut self) -> World {
        World::new(&mut self.universe)
    }

    /// Effectively "clears a world". Does a quick pop & push of a clean slate world.
    pub fn clear(&mut self) {
        self.pop();
        let new_world = self.create();
        self.push(new_world);
    }

    /// Pops off the most recently added world.
    pub fn pop(&mut self) -> Option<World> { self.world_stack.pop() }

    /// Pushes a world on top of the world stack, this will be the new active world.
    pub fn push(&mut self, world: World) { self.world_stack.push(world) }

    /// Insert entities into the current world.
    pub fn insert<T, C>(&mut self, tags: T, components: C) -> &[Entity]
    where
        T: TagSet + TagLayout + for<'a> Filter<ChunksetFilterData<'a>>,
        C: IntoComponentSource,
    { self.world().inner.insert(tags, components) }

    pub fn queryable(&mut self) -> &mut legion::world::World { &mut self.world().inner }
    pub fn queryable_ref(&self) -> &legion::world::World { &self.world_ref().inner }

    /// Get a reference to the current active world.
    pub fn world_ref(&self) -> &World {
        if self.world_stack.len() == 0 {
            panic!("There are no worlds available to process.");
        }

        &self.world_stack[self.world_stack.len() - 1]
    }

    /// Get a mutable reference to the current active world.
    pub fn world(&mut self) -> &mut World {
        if self.world_stack.len() == 0 {
            panic!("There are no worlds available to process.");
        }

        let i = self.world_stack.len() - 1;
        self.world_stack.get_mut(i).unwrap()
    }

    pub fn physics(&mut self) -> PhysicsHandler {
        let world = self.world();

        PhysicsHandler::new(&mut world.physics_engine, &mut world.inner)
    }
}