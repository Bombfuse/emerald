use crate::world::*;

pub struct Emerald<'a> {
    world_engine: &'a mut WorldEngine,
}
impl<'a> Emerald<'a> {
    pub fn new(world_engine: &'a mut WorldEngine) -> Self {
        Emerald {
            world_engine,
        }
    }


    /// World
    pub fn world(&mut self) -> WorldHandler {
        WorldHandler::new(&mut self.world_engine)
    }

    pub fn create_world(&mut self) -> World {
        World::new()
    }

    pub fn push_world(&mut self, world: World) { self.world_engine.push(world) }
    pub fn pop_world(&mut self, world: World) -> World { self.world_engine.pop().unwrap() }
}