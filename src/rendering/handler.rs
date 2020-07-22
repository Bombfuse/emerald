use crate::{RenderingEngine, WorldEngine, Rectangle};
use miniquad::Context;

pub struct GraphicsHandler<'a> {
    quad_ctx: &'a mut Context,
    rendering_engine: &'a mut RenderingEngine,
    world_engine: &'a mut WorldEngine,
}
impl<'a> GraphicsHandler<'a> {
    pub fn new(quad_ctx: &'a mut Context,
        rendering_engine: &'a mut RenderingEngine,
        world_engine: &'a mut WorldEngine
    ) -> Self {
        GraphicsHandler {
            quad_ctx,
            rendering_engine,
            world_engine
        }
    }

    pub fn draw_world(&mut self) {
        self.rendering_engine.draw_world(&mut self.quad_ctx, self.world_engine.world())
    }

    pub fn set_projection(&mut self, projection: Rectangle) {
        self.rendering_engine.projection = projection;
    }

    /// Begin drawing to the screen
    pub fn begin(&mut self) {
        self.rendering_engine.begin(&mut self.quad_ctx);
    }

    /// Commit all drawings to the screen
    pub fn render(&mut self) {
        self.rendering_engine.render(&mut self.quad_ctx);
    }
}