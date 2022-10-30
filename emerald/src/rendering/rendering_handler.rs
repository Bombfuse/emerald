use crate::{
    rendering_engine::{DrawCommand, Drawable, RenderingEngine},
    texture::TextureKey,
    AssetStore, EmeraldError, Transform, World,
};

use super::components::{ColorRect, Label, Sprite};

pub struct RenderingHandler<'c> {
    asset_store: &'c mut AssetStore,
    rendering_engine: &'c mut RenderingEngine,
}
impl<'c> RenderingHandler<'c> {
    pub(crate) fn new(
        asset_store: &'c mut AssetStore,
        rendering_engine: &'c mut RenderingEngine,
    ) -> Self {
        RenderingHandler {
            asset_store,
            rendering_engine,
        }
    }

    pub fn draw_world(&mut self, world: &mut World) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_world(world, &mut self.asset_store)
    }

    pub fn draw_colliders(
        &mut self,
        world: &mut World,
        color: crate::Color,
    ) -> Result<(), EmeraldError> {
        self.rendering_engine.draw_colliders(world, color)
    }

    pub fn draw_sprite(
        &mut self,
        sprite: &Sprite,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        self.rendering_engine.push_draw_command(
            &mut self.asset_store,
            DrawCommand {
                drawable: Drawable::Sprite {
                    sprite: sprite.clone(),
                },
                transform: *transform,
                z_index: sprite.z_index,
            },
        )
    }

    pub fn draw_label(&mut self, label: &Label, transform: &Transform) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_label(&mut self.asset_store, label, transform)
    }

    pub fn draw_color_rect(
        &mut self,
        color_rect: &ColorRect,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        self.rendering_engine.push_draw_command(
            &mut self.asset_store,
            DrawCommand {
                drawable: Drawable::ColorRect {
                    color_rect: *color_rect,
                },
                transform: *transform,
                z_index: color_rect.z_index,
            },
        )
    }

    /// Begin drawing to the screen
    pub fn begin(&mut self) -> Result<(), EmeraldError> {
        self.rendering_engine.begin(&mut self.asset_store)
    }

    /// Begin drawing to the screen
    pub fn begin_texture(&mut self, texture_key: TextureKey) -> Result<(), EmeraldError> {
        self.rendering_engine
            .begin_texture(texture_key, &mut self.asset_store)
    }

    /// Commit all drawings to the screen
    pub fn render(&mut self) -> Result<(), EmeraldError> {
        self.rendering_engine.render(&mut self.asset_store)
    }

    /// Commit all drawings to the screen
    pub fn render_texture(&mut self) -> Result<TextureKey, EmeraldError> {
        self.rendering_engine.render_texture(&mut self.asset_store)
    }

    pub fn set_fullscreen(&mut self, fs: bool) -> Result<(), EmeraldError> {
        todo!();
        Ok(())
    }

    pub fn set_window_size(&mut self, x: u32, y: u32) -> Result<(), EmeraldError> {
        todo!();
        Ok(())
    }
}
