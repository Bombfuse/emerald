use crate::world::*;
use crate::assets::*;
use crate::rendering::*;

pub struct Emerald<'a> {
    quad_ctx: &'a mut miniquad::Context,
    rendering_engine: &'a mut RenderingEngine,
    world_engine: &'a mut WorldEngine,
}
impl<'a> Emerald<'a> {
    pub fn new(quad_ctx: &'a mut miniquad::Context, world_engine: &'a mut WorldEngine, rendering_engine: &'a mut RenderingEngine) -> Self {
        Emerald {
            quad_ctx,
            rendering_engine,
            world_engine,
        }
    }

    /// Asset loading
    pub fn loader(&mut self) -> AssetLoader {
        AssetLoader::new(&mut self.quad_ctx, &mut self.rendering_engine)
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