use alloc::{boxed::Box, vec::Vec};

use crate::{
    asset_key::AssetKey, game_engine::GameEngineContext, math::Vector2,
    rendering_engine::RenderingEngine, AssetEngine, Color, EmeraldError, Transform, World,
};

use super::components::{ColorRect, ColorTri, Label, Sprite};

pub struct RenderingHandler<'c> {
    asset_engine: &'c mut AssetEngine,
    rendering_engine: &'c mut Box<dyn RenderingEngine>,
    ctx: &'c mut GameEngineContext,
}
impl<'c> RenderingHandler<'c> {
    pub(crate) fn new(
        asset_engine: &'c mut AssetEngine,
        rendering_engine: &'c mut Box<dyn RenderingEngine>,
        ctx: &'c mut GameEngineContext,
    ) -> Self {
        RenderingHandler {
            asset_engine,
            rendering_engine,
            ctx,
        }
    }

    pub fn draw_world(&mut self, world: &mut World) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_world(world, &mut self.asset_engine)
    }

    /// Draws the world with the given transform applied to the active camera.
    pub fn draw_world_with_transform(
        &mut self,
        world: &mut World,
        transform: Transform,
    ) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_world_with_transform(world, transform, &mut self.asset_engine)
    }

    pub fn draw_sprite(
        &mut self,
        sprite: &Sprite,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_sprite(&mut self.asset_engine, sprite, transform)
    }

    pub fn draw_label(&mut self, label: &Label, transform: &Transform) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_label(&mut self.asset_engine, label, transform)
    }

    /// Draw a triangle with the given points at the given transform.
    /// Points are drawn with the given transform as an offset.
    pub fn draw_color_tri(
        &mut self,
        color_tri: &ColorTri,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_color_tri(&mut self.asset_engine, color_tri, transform)
    }

    pub fn draw_color_rect(
        &mut self,
        color_rect: &ColorRect,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_color_rect(&mut self.asset_engine, color_rect, transform)
    }

    /// Begin drawing to the screen
    pub fn begin(&mut self) -> Result<(), EmeraldError> {
        self.rendering_engine.begin(&mut self.asset_engine)
    }

    /// Begin drawing to the screen
    pub fn begin_texture(&mut self, texture_key: &AssetKey) -> Result<(), EmeraldError> {
        self.rendering_engine
            .begin_texture(texture_key, &mut self.asset_engine)
    }

    /// Commit all drawings to the screen
    pub fn render(&mut self) -> Result<(), EmeraldError> {
        self.rendering_engine.render(&mut self.asset_engine)
    }

    /// Commit all drawings to the active key
    pub fn render_texture(&mut self) -> Result<(), EmeraldError> {
        self.rendering_engine.render_texture(&mut self.asset_engine)
    }

    pub fn set_fullscreen(&mut self, fs: bool) -> Result<(), EmeraldError> {
        Ok(())
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) -> Result<(), EmeraldError> {
        Ok(())
    }
}
