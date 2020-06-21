use crate::world::*;
use crate::assets::*;
use crate::rendering::*;
use crate::input::*;
use crate::logging::*;

pub struct Emerald<'a> {
    quad_ctx: &'a mut miniquad::Context,
    rendering_engine: &'a mut RenderingEngine,
    logging_engine: &'a mut LoggingEngine,
    input_engine: &'a mut InputEngine,
    world_engine: &'a mut WorldEngine,
}
impl<'a> Emerald<'a> {
    pub(crate) fn new(quad_ctx: &'a mut miniquad::Context,
        input_engine: &'a mut InputEngine,
        world_engine: &'a mut WorldEngine,
        logging_engine: &'a mut LoggingEngine,
        rendering_engine: &'a mut RenderingEngine) -> Self {

        Emerald {
            quad_ctx,
            rendering_engine,
            input_engine,
            logging_engine,
            world_engine,
        }
    }

    /// Asset loading
    pub fn loader(&mut self) -> AssetLoader {
        AssetLoader::new(&mut self.quad_ctx, &mut self.rendering_engine)
    }

    /// Logging
    pub fn logger(&mut self) -> LoggingHandler {
        LoggingHandler::new(&mut self.logging_engine)
    }

    /// Input
    pub fn input(&mut self) -> InputHandler {
        InputHandler::new(&mut self.input_engine)
    }

    /// World
    pub fn world(&mut self) -> &mut WorldEngine {
        &mut self.world_engine
    }

    pub fn push_world(&mut self, world: World) { self.world_engine.push(world) }
    pub fn pop_world(&mut self, world: World) -> World { self.world_engine.pop().unwrap() }
}