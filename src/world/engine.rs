use crate::world::*;

use legion::prelude::{Universe};

pub struct WorldEngine {
    world_stack: Vec<World>,
    universe: Universe,
}
impl WorldEngine {
    pub fn new() -> Self {
        let universe = Universe::new();

        WorldEngine {
            world_stack: Vec::new(),
            universe,
        }
    }

    pub fn pop(&mut self) -> Option<World> { self.world_stack.pop() }

    pub fn push(&mut self, world: World) { self.world_stack.push(world) }

    pub fn create_world(&mut self) -> World {
        World::new(&mut self.universe)
    }

    pub fn world(&self) -> &World {
        if self.world_stack.len() == 0 {
            panic!("There are no worlds available to process.");
        }

        &self.world_stack[self.world_stack.len() - 1]
    }

    pub fn world_mut(&mut self) -> &mut World {
        if self.world_stack.len() == 0 {
            panic!("There are no worlds available to process.");
        }

        let i = self.world_stack.len() - 1;
        self.world_stack.get_mut(i).unwrap()
    }
}