use crate::world::*;
use crate::assets::*;

pub struct Emerald<'a> {
    world_engine: &'a mut WorldEngine,
}
impl<'a> Emerald<'a> {
    pub fn new(world_engine: &'a mut WorldEngine) -> Self {
        Emerald {
            world_engine,
        }
    }

    /// Asset loading
    pub fn loader(&mut self) -> AssetLoader {
        AssetLoader::new()
    }

    /// World
    pub fn world(&mut self) -> &mut World {
        self.world_engine.world_mut()
    }
    
    pub fn world_borrow(&mut self) -> &World {
        self.world_engine.world()
    }

    pub fn create_world(&mut self) -> World {
        self.world_engine.create_world()
    }

    pub fn push_world(&mut self, world: World) { self.world_engine.push(world) }
    pub fn pop_world(&mut self, world: World) -> World { self.world_engine.pop().unwrap() }
}