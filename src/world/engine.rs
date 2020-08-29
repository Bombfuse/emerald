use crate::world::*;
use crate::world::physics::*;

pub struct WorldEngine {
    world_stack: Vec<EmeraldWorld>,
}
impl WorldEngine {
    pub(crate) fn new() -> Self {
        WorldEngine {
            world_stack: Vec::new(),
        }
    }

    /// Public API ///
    /// Pops off the most recently added world.
    pub fn pop_world(&mut self) -> Option<EmeraldWorld> { self.world_stack.pop() }

    /// Pushes a world on top of the world stack, this will be the new active world.
    pub fn push_world(&mut self, world: EmeraldWorld) { self.world_stack.push(world) }

    /// Returns the inner legion world, exposing the real world API.
    pub fn inner(&mut self) -> &mut hecs::World { &mut self.world().inner }
    pub fn inner_ref(&self) -> &hecs::World { &self.world_ref().inner }

    /// Get a reference to the current active world.
    pub(crate) fn world_ref(&self) -> &EmeraldWorld {
        if self.world_stack.len() == 0 {
            panic!("There are no worlds available to process.");
        }

        &self.world_stack[self.world_stack.len() - 1]
    }

    /// Get a mutable reference to the current active world.
    pub(crate) fn world(&mut self) -> &mut EmeraldWorld {
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