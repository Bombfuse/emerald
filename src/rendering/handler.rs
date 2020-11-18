use crate::*;
use miniquad::Context;

pub struct GraphicsHandler<'a> {
    quad_ctx: &'a mut Context,
    rendering_engine: &'a mut RenderingEngine,
    world_engine: &'a mut WorldEngine,
}
impl<'a> GraphicsHandler<'a> {
    pub fn new(
        quad_ctx: &'a mut Context,
        rendering_engine: &'a mut RenderingEngine,
        world_engine: &'a mut WorldEngine,
    ) -> Self {
        GraphicsHandler {
            quad_ctx,
            rendering_engine,
            world_engine,
        }
    }
    
    pub fn draw_world(&mut self) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_world(&mut self.quad_ctx, self.world_engine.world())
    }

    pub fn draw_colliders(&mut self, color: crate::Color) {
        self.rendering_engine
            .draw_colliders(&mut self.quad_ctx, self.world_engine.world(), color)
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite, pos: &Position) {
        self.rendering_engine
            .draw_sprite(&mut self.quad_ctx, sprite, pos)
    }

    pub fn draw_label(&mut self, label: &Label, pos: &Position) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_label(&mut self.quad_ctx, label, pos)
    }

    pub fn draw_color_rect(&mut self, color_rect: &ColorRect, pos: &Position) {
        self.rendering_engine
            .draw_color_rect(&mut self.quad_ctx, color_rect, pos)
    }

    /// Begin drawing to the screen
    pub fn begin(&mut self) {
        self.rendering_engine.begin(&mut self.quad_ctx);
    }

    /// Render to a sprite this pass
    pub fn begin_sprite(&mut self) {
        self.rendering_engine.begin_texture(&mut self.quad_ctx)
    }

    /// Commit all drawings to the screen
    pub fn render(&mut self) {
        self.rendering_engine.render(&mut self.quad_ctx);
    }

    /// Finish rendering to a texture and hand it back to the user
    pub fn render_texture(&mut self) -> Result<Texture, EmeraldError> {
        self.rendering_engine.render_to_texture(&mut self.quad_ctx)
    }
}
