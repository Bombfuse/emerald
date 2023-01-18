use rapier2d::na::Vector2;

use crate::{
    game_engine::GameEngineContext,
    rendering_engine::{DrawCommand, Drawable, RenderingEngine},
    texture::TextureKey,
    AssetStore, EmeraldError, Transform, World,
};

use super::components::{ColorRect, ColorTri, Label, Sprite};

pub struct RenderingHandler<'c> {
    asset_store: &'c mut AssetStore,
    rendering_engine: &'c mut RenderingEngine,
    ctx: &'c mut GameEngineContext,
}
impl<'c> RenderingHandler<'c> {
    pub(crate) fn new(
        asset_store: &'c mut AssetStore,
        rendering_engine: &'c mut RenderingEngine,
        ctx: &'c mut GameEngineContext,
    ) -> Self {
        RenderingHandler {
            asset_store,
            rendering_engine,
            ctx,
        }
    }

    pub fn draw_world(&mut self, world: &mut World) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_world(world, &mut self.asset_store)
    }

    /// Draws the world with the given transform applied to the active camera.
    pub fn draw_world_with_transform(
        &mut self,
        world: &mut World,
        transform: Transform,
    ) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_world_with_transform(world, transform, &mut self.asset_store)
    }

    pub fn draw_colliders(
        &mut self,
        world: &mut World,
        color: crate::Color,
    ) -> Result<(), EmeraldError> {
        self.rendering_engine
            .draw_colliders(&mut self.asset_store, world, color)
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

    /// Draw a triangle with the given points at the given transform.
    /// Points are drawn with the given transform as an offset.
    pub fn draw_color_tri(
        &mut self,
        color_tri: &ColorTri,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        self.rendering_engine.push_draw_command(
            &mut self.asset_store,
            DrawCommand {
                drawable: Drawable::ColorTri {
                    color_tri: color_tri.clone(),
                },
                transform: transform.clone(),
                z_index: color_tri.z_index,
            },
        )
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
        if let Some(window) = &mut self.ctx.window {
            if fs {
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            } else {
                window.set_fullscreen(None);
            };
        }
        Ok(())
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) -> Result<(), EmeraldError> {
        if let Some(window) = &mut self.ctx.window {
            let size = winit::dpi::PhysicalSize::new(width, height);
            window.set_inner_size(size);
        }

        Ok(())
    }
}
