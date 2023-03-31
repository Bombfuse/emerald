use rapier2d::{
    na::Vector2,
    prelude::{ConvexPolygon, RigidBodyHandle},
};

use crate::{
    game_engine::GameEngineContext, rendering_engine::RenderingEngine, texture::TextureKey,
    AssetEngine, Color, EmeraldError, Transform, World,
};

use super::components::{ColorRect, ColorTri, Label, Sprite};

pub struct RenderingHandler<'c> {
    asset_engine: &'c mut AssetEngine,
    rendering_engine: &'c mut RenderingEngine,
    ctx: &'c mut GameEngineContext,
}
impl<'c> RenderingHandler<'c> {
    pub(crate) fn new(
        asset_engine: &'c mut AssetEngine,
        rendering_engine: &'c mut RenderingEngine,
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

    pub fn draw_colliders(
        &mut self,
        world: &mut World,
        color: crate::Color,
    ) -> Result<(), EmeraldError> {
        if world.physics_engine.is_none() {
            return Ok(());
        }

        let mut color_rect = ColorRect {
            color,
            ..Default::default()
        };

        let camera_transform = if let Some(e) = world.get_active_camera() {
            world.get::<&mut Transform>(e)?.clone()
        } else {
            Default::default()
        };

        for (_id, (body_handle, transform)) in
            world.inner.query::<(&RigidBodyHandle, &Transform)>().iter()
        {
            if let Some(body) = world
                .physics_engine
                .as_ref()
                .unwrap()
                .bodies
                .get(*body_handle)
            {
                for collider_handle in body.colliders() {
                    if let Some(collider) = world
                        .physics_engine
                        .as_ref()
                        .unwrap()
                        .colliders
                        .get(*collider_handle)
                    {
                        let offset = collider.translation() - body.translation();
                        let collider_transform = transform.clone()
                            + Transform::from_translation((offset.x, offset.y))
                            - camera_transform;

                        if let Some(polygon) = collider.shape().as_convex_polygon() {
                            self.draw_convex_polygon(&color, &polygon, &collider_transform)?;
                            continue;
                        }

                        if let Some(cuboid) = collider.shape().as_cuboid() {
                            color_rect.width = cuboid.half_extents.x as u32 * 2;
                            color_rect.height = cuboid.half_extents.y as u32 * 2;
                            self.draw_color_rect(&color_rect, &collider_transform)?;
                            continue;
                        }

                        // If we dont have specific logic for the shape let's just draw its AABB
                        let aabb = collider.compute_aabb();
                        color_rect.width = aabb.half_extents().x as u32 * 2;
                        color_rect.height = aabb.half_extents().y as u32 * 2;
                        self.draw_color_rect(&color_rect, &collider_transform)?;
                    }
                }
            }
        }

        Ok(())
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

    pub fn draw_convex_polygon(
        &mut self,
        color: &Color,
        convex_polygon: &ConvexPolygon,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        let vertices = convex_polygon
            .points()
            .iter()
            .flat_map(|p| [p.x as f64, p.y as f64])
            .collect::<Vec<f64>>();
        let triangles = emd_earcutr::earcut(&vertices, &vec![], 2);
        for tri in triangles.chunks_exact(3) {
            let i1 = tri[0] * 2;
            let i2 = tri[1] * 2;
            let i3 = tri[2] * 2;
            let color_tri = ColorTri::new(
                color.clone(),
                [
                    Vector2::new(vertices[i1] as f32, vertices[i1 + 1] as f32),
                    Vector2::new(vertices[i2] as f32, vertices[i2 + 1] as f32),
                    Vector2::new(vertices[i3] as f32, vertices[i3 + 1] as f32),
                ],
            );
            self.draw_color_tri(&color_tri, transform)?;
        }

        Ok(())
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
    pub fn begin_texture(&mut self, texture_key: &TextureKey) -> Result<(), EmeraldError> {
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
