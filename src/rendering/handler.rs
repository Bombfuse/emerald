use crate::*;
use miniquad::Context;

pub struct GraphicsHandler<'a> {
    quad_ctx: &'a mut Context,
    rendering_engine: &'a mut RenderingEngine,
}
impl<'a> GraphicsHandler<'a> {
    pub fn new(
        quad_ctx: &'a mut Context,
        rendering_engine: &'a mut RenderingEngine,
    ) -> Self {
        GraphicsHandler {
            quad_ctx,
            rendering_engine,
        }
    }

    pub fn draw_world(&mut self, world: &mut EmeraldWorld) -> Result<(), EmeraldError> {
        self.rendering_engine.draw_world(&mut self.quad_ctx, world)
    }

    #[cfg(feature = "physics")]
    pub fn draw_colliders(&mut self, world: &mut EmeraldWorld, color: crate::Color) {
        self.rendering_engine
            .draw_colliders(&mut self.quad_ctx, world, color)
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

    /// Commit all drawings to the screen
    pub fn render(&mut self) {
        self.rendering_engine.render(&mut self.quad_ctx);
    }
}
