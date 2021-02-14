use crate::rendering::components::*;
use crate::rendering::*;
use crate::world::*;
use crate::*;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use glam::{Mat4, Vec2, Vec4};
use miniquad::*;

pub(crate) struct RenderingEngine {
    pub(crate) settings: RenderSettings,
    pipeline: Pipeline,
    layout: Layout,
}
impl RenderingEngine {
    pub(crate) fn new(ctx: &mut Context, settings: RenderSettings) -> Self {
        let shader = Shader::new(ctx, VERTEX, FRAGMENT, shaders::meta()).unwrap();

        let mut params = PipelineParams::default();
        params.depth_write = true;
        params.color_blend = Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        ));
        params.alpha_blend = Some(BlendState::new(
            Equation::Add,
            BlendFactor::Zero,
            BlendFactor::One,
        ));

        let pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[VertexAttribute::new("position", VertexFormat::Float2)],
            shader,
            params,
        );

        RenderingEngine {
            settings,
            pipeline,
            layout: Layout::new(CoordinateSystem::PositiveYDown),
        }
    }

    #[inline]
    pub fn draw_world(
        &mut self,
        mut ctx: &mut Context,
        asset_store: &mut AssetStore,
        world: &mut EmeraldWorld,
    ) -> Result<(), EmeraldError> {
        let screen_size = self.get_screen_size(ctx);
        let (camera, camera_position) = get_camera_and_camera_position(world);
        let mut draw_queue = Vec::new();

        for (_id, (aseprite, position)) in world.inner.query::<(&mut Aseprite, &Position)>().iter()
        {
            aseprite.update();

            if is_in_view(&aseprite.sprite, &position, &camera, &screen_size) {
                let drawable = Drawable::Aseprite {
                    sprite: aseprite.sprite.clone(),
                    offset: aseprite.offset.clone(),
                    scale: aseprite.scale.clone(),
                    centered: aseprite.centered,
                    color: aseprite.color.clone(),
                    rotation: aseprite.rotation,
                    z_index: aseprite.z_index,
                    visible: aseprite.visible,
                };

                draw_queue.push(DrawCommand {
                    drawable,
                    position: position.clone(),
                    z_index: aseprite.z_index,
                });
            }
        }

        for (_id, (sprite, position)) in world.inner.query::<(&Sprite, &Position)>().iter() {
            if is_in_view(&sprite, &position, &camera, &screen_size) {
                let drawable = Drawable::Sprite {
                    sprite: sprite.clone(),
                };

                draw_queue.push(DrawCommand {
                    drawable,
                    position: position.clone(),
                    z_index: sprite.z_index,
                });
            }
        }

        for (_id, (color_rect, position)) in world.inner.query::<(&ColorRect, &Position)>().iter() {
            let drawable = Drawable::ColorRect {
                color_rect: color_rect.clone(),
            };

            draw_queue.push(DrawCommand {
                drawable,
                position: position.clone(),
                z_index: color_rect.z_index,
            });
        }

        for (_, (label, position)) in world.query::<(&Label, &Position)>().iter() {
            let drawable = Drawable::Label {
                label: label.clone(),
            };

            draw_queue.push(DrawCommand {
                drawable,
                position: position.clone(),
                z_index: label.z_index,
            })
        }

        draw_queue.sort_by(|a, b| a.z_index.partial_cmp(&b.z_index).unwrap());

        for draw_command in draw_queue {
            let position = {
                let mut pos = draw_command.position.clone() - camera_position;

                if camera.centered {
                    pos = pos + Position::new(screen_size.0 / 2.0, screen_size.1 / 2.0);
                }

                pos.x += camera.offset.x;
                pos.y += camera.offset.y;

                pos
            };

            match draw_command.drawable {
                Drawable::Aseprite {
                    sprite,
                    rotation,
                    offset,
                    centered,
                    visible,
                    scale,
                    color,
                    z_index,
                } => self.draw_aseprite(
                    &mut ctx, asset_store, &sprite, rotation, &offset, centered, visible, &scale, &color,
                    z_index, &position,
                ),
                Drawable::Sprite { sprite } => self.draw_sprite(&mut ctx, asset_store, &sprite, &position),
                Drawable::ColorRect { color_rect } => {
                    self.draw_color_rect(&mut ctx, asset_store, &color_rect, &position)
                }
                Drawable::Label { label } => self.draw_label(&mut ctx, asset_store, &label, &position)?,
            }
        }

        ctx.end_render_pass();

        Ok(())
    }

    #[inline]
    #[cfg(feature = "physics")]
    pub fn draw_colliders(
        &mut self,
        mut ctx: &mut Context,
        asset_store: &mut AssetStore,
        world: &mut EmeraldWorld,
        collider_color: Color,
    ) {
        let screen_size = self.get_screen_size(ctx);
        let mut color_rect = ColorRect::default();
        color_rect.color = collider_color;
        let (camera, camera_position) = get_camera_and_camera_position(world);

        for (_id, body_handle) in world.inner.query::<&RigidBodyHandle>().iter() {
            if let Some(body) = world.physics_engine.bodies.get(*body_handle) {
                for collider_handle in body.colliders() {
                    if let Some(collider) =
                        world.physics_engine.colliders.get(collider_handle.clone())
                    {
                        let aabb = collider.compute_aabb();
                        let pos = Position::new(aabb.center().coords.x, aabb.center().coords.y);
                        color_rect.width = aabb.half_extents().x as u32 * 2;
                        color_rect.height = aabb.half_extents().y as u32 * 2;

                        let position = {
                            let mut pos = pos - camera_position;

                            if camera.centered {
                                pos = pos + Position::new(screen_size.0 / 2.0, screen_size.1 / 2.0);
                            }

                            pos
                        };

                        self.draw_color_rect(&mut ctx, asset_store, &color_rect, &position);
                    }
                }
            }
        }
    }

    #[inline]
    pub(crate) fn begin(&mut self, ctx: &mut Context) {
        ctx.begin_default_pass(PassAction::Clear {
            color: Some(self.settings.background_color.to_percentage()),
            depth: None,
            stencil: None,
        });
    }

    #[inline]
    pub(crate) fn render(&mut self, ctx: &mut Context) {
        ctx.end_render_pass();
        ctx.commit_frame();
    }

    pub(crate) fn draw_label(
        &mut self,
        mut ctx: &mut Context,
        mut asset_store: &mut AssetStore,
        label: &Label,
        position: &Position,
    ) -> Result<(), EmeraldError> {
        self.layout.reset(&LayoutSettings {
            max_width: label.max_width,
            max_height: label.max_height,
            wrap_style: label.wrap_style,
            horizontal_align: label.horizontal_align,
            vertical_align: label.vertical_align,
            ..LayoutSettings::default()
        });

        if let Some(font) = asset_store.get_fontdue_font(&label.font_key) {
            self.layout.append(&[font], &TextStyle::new(&label.text, label.font_size as f32, 0));
        }

        let mut font_texture_width = 0;
        let mut font_texture_height = 0;
        let mut font_texture_key: Option<TextureKey> = None;

        if let Some(font) = asset_store.get_font_mut(&label.font_key) {
            font_texture_key = Some(font.font_texture_key.clone());
        }

        if let Some(font_texture_key) = font_texture_key.as_ref() {
            if let Some(texture) = asset_store.get_texture(&font_texture_key) {
                font_texture_width = texture.width;
                font_texture_height = texture.height;
            }
        }

        let mut draw_calls: Vec<(
            f32,       // z_index
            Vec2,      // real_scale
            Vec2,      // real_position
            Rectangle, // target
            Color,     // color
            bool,      // centered
            bool,      // Visible
            Option<f32>, // max_width
        )> = Vec::new();

        ctx.apply_pipeline(&self.pipeline);

        let mut remaining_char_count = label.visible_characters;
        if label.visible_characters < 0 {
            remaining_char_count = label.text.len() as i64;
        }

        for glyph in self.layout.glyphs() {
            let character = glyph.key.c;
            let x = glyph.x;
            let y = glyph.y;

            let mut need_to_cache_glyph = false;
            if let Some(font) = asset_store.get_font(&label.font_key) {
                need_to_cache_glyph = !font.characters.contains_key(&(character, label.font_size));
            }
            
            if need_to_cache_glyph {
                cache_glyph(&mut ctx, &mut asset_store, &label.font_key, character, label.font_size)?;
            }

            if let Some(font) = asset_store.get_font_mut(&label.font_key) {
                let font_data = &font.characters[&(character, label.font_size)];
                {
                    let left_coord = (font_data.offset_x as f32 + x) * label.scale;
                    let top_coord = y * label.scale;

                    let target = Rectangle::new(
                        (font_data.glyph_x as f32) / font_texture_width as f32,
                        (font_data.glyph_y as f32) / font_texture_height as f32,
                        (font_data.glyph_w as f32) / font_texture_width as f32,
                        (font_data.glyph_h as f32) / font_texture_height as f32,
                    );

                    let real_scale = Vec2::new(
                        label.scale * target.width * font_texture_width as f32,
                        label.scale * target.height * font_texture_height as f32 * -1.0,
                    );
                    let real_position = Vec2::new(
                        position.x + label.offset.x + left_coord,
                        position.y - label.offset.y - top_coord,
                    );

                    if remaining_char_count > 0 {
                        draw_calls.push((
                            label.z_index,
                            real_scale,
                            real_position,
                            target,
                            label.color,
                            label.centered,
                            label.visible,
                            label.max_width.clone(),
                        ));
                    }
                }

                remaining_char_count -= 1;
            }
        }

        if let Some(font_texture_key) = font_texture_key {
            for draw_call in draw_calls {
                let (z_index, real_scale, mut real_position, target, mut color, centered, visible, max_width) = draw_call;
    
                if centered {
                    if let Some(max_width) = max_width {
                        real_position.x -= max_width / 2.0;
                    }
                }

                if !visible {
                    color.a = 0;
                }
    
                draw_texture(
                    &self.settings,
                    &mut ctx,
                    &mut asset_store,
                    &font_texture_key,
                    z_index,
                    real_scale,
                    0.0,
                    Vec2::new(0.0, 0.0),
                    real_position,
                    target,
                    color,
                );
            }
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn draw_color_rect(
        &mut self,
        mut ctx: &mut Context,
        mut asset_store: &mut AssetStore,
        color_rect: &ColorRect,
        position: &Position,
    ) {
        ctx.apply_pipeline(&self.pipeline);

        let (width, height) = (color_rect.width, color_rect.height);
        let mut offset = color_rect.offset.clone();

        if color_rect.centered {
            offset.x -= (color_rect.width / 2) as f32;
            offset.y -= (color_rect.height / 2) as f32;
        }

        let real_scale = Vec2::new(width as f32, height as f32);
        let real_position = Vec2::new(position.x + offset.x, position.y + offset.y);

        draw_texture(
            &self.settings,
            &mut ctx,
            &mut asset_store,
            &TextureKey::default(),
            color_rect.z_index,
            real_scale,
            color_rect.rotation,
            Vec2::new(0.0, 0.0),
            real_position,
            Rectangle::new(0.0, 0.0, 1.0, 1.0),
            color_rect.color,
        )
    }

    #[inline]
    pub(crate) fn draw_aseprite(
        &mut self,
        mut ctx: &mut Context,
        mut asset_store: &mut AssetStore,
        sprite: &Sprite,
        rotation: f32,
        offset: &Vector2<f32>,
        centered: bool,
        visible: bool,
        scale: &Vector2<f32>,
        color: &Color,
        z_index: f32,
        position: &Position,
    ) {
        if !visible {
            return;
        }

        ctx.apply_pipeline(&self.pipeline);
        let texture = asset_store.get_texture(&sprite.texture_key).unwrap();
        let mut target = Rectangle::new(
            sprite.target.x / texture.width as f32,
            sprite.target.y / texture.height as f32,
            sprite.target.width / texture.width as f32,
            sprite.target.height / texture.height as f32,
        );

        if sprite.target.is_zero_sized() {
            target = Rectangle::new(0.0, 0.0, 1.0, 1.0);
        }

        let mut offset = offset.clone();
        if centered {
            if sprite.target.is_zero_sized() {
                offset.x -= scale.x * texture.width as f32 / 2.0;
                offset.y -= scale.y * texture.height as f32 / 2.0;
            } else {
                offset.x -= scale.x * sprite.target.width / 2.0;
                offset.y -= scale.y * sprite.target.height / 2.0;
            }
        }

        let real_scale = Vec2::new(
            scale.x * target.width * (f32::from(texture.width)),
            scale.y * target.height * (f32::from(texture.height)),
        );
        let real_position = Vec2::new(position.x + offset.x, position.y + offset.y);

        draw_texture(
            &self.settings,
            &mut ctx,
            &mut asset_store,
            &sprite.texture_key,
            z_index,
            real_scale,
            rotation,
            Vec2::new(0.0, 0.0),
            real_position,
            target,
            color.clone(),
        )
    }

    #[inline]
    pub(crate) fn draw_sprite(
        &mut self,
        mut ctx: &mut Context,
        mut asset_store: &mut AssetStore,
        sprite: &Sprite,
        position: &Position,
    ) {
        if !sprite.visible {
            return;
        }

        ctx.apply_pipeline(&self.pipeline);
        let texture = asset_store.get_texture(&sprite.texture_key).unwrap();
        let mut target = Rectangle::new(
            sprite.target.x / texture.width as f32,
            sprite.target.y / texture.height as f32,
            sprite.target.width / texture.width as f32,
            sprite.target.height / texture.height as f32,
        );

        if sprite.target.is_zero_sized() {
            target = Rectangle::new(0.0, 0.0, 1.0, 1.0);
        }

        let mut offset = sprite.offset.clone();
        if sprite.centered {
            if sprite.target.is_zero_sized() {
                offset.x -= sprite.scale.x * texture.width as f32 / 2.0;
                offset.y -= sprite.scale.y * texture.height as f32 / 2.0;
            } else {
                offset.x -= sprite.scale.x * sprite.target.width / 2.0;
                offset.y -= sprite.scale.y * sprite.target.height / 2.0;
            }
        }

        let real_scale = Vec2::new(
            sprite.scale.x * target.width * (f32::from(texture.width)),
            sprite.scale.y * target.height * (f32::from(texture.height)),
        );
        let real_position = Vec2::new(position.x + offset.x, position.y + offset.y);

        draw_texture(
            &self.settings,
            &mut ctx,
            &mut asset_store,
            &sprite.texture_key,
            sprite.z_index,
            real_scale,
            sprite.rotation,
            Vec2::new(0.0, 0.0),
            real_position,
            target,
            sprite.color.clone(),
        )
    }

    #[inline]
    fn get_screen_size(&self, ctx: &Context) -> (f32, f32) {
        match self.settings.scalar {
            ScreenScalar::None => ctx.screen_size(),
            _ => (
                self.settings.resolution.0 as f32,
                self.settings.resolution.1 as f32,
            ),
        }
    }
}

#[inline]
fn draw_texture(
    settings: &RenderSettings,
    mut ctx: &mut Context,
    asset_store: &mut AssetStore,
    texture_key: &TextureKey,
    _z_index: f32,
    scale: Vec2,
    rotation: f32,
    offset: Vec2,
    position: Vec2,
    source: Rectangle,
    color: Color,
) {
    let view_size = ctx.screen_size();
    let mut uniforms = Uniforms::default();

    let projection = match settings.scalar {
        ScreenScalar::Stretch => Mat4::orthographic_rh_gl(
            0.0,
            settings.resolution.0 as f32,
            0.0,
            settings.resolution.1 as f32,
            -1.0,
            1.0,
        ),
        ScreenScalar::None => {
            Mat4::orthographic_rh_gl(0.0, view_size.0, 0.0, view_size.1, -1.0, 1.0)
        }
        ScreenScalar::Keep => {
            let x_start = 0.0;
            let y_start = 0.0;
            let x_end = settings.resolution.0 as f32;
            let y_end = settings.resolution.1 as f32;
            // let keep_height = (view_size.0 * settings.resolution.1 as f32) > (view_size.1 * settings.resolution.0 as f32);

            // if keep_height {
            //     let scale = view_size.1 / settings.resolution.1 as f32;
            //     let width = settings.resolution.0 as f32 * scale;
            //     x_start = (view_size.0 - width) / 2.0;
            //     x_end = (view_size.0 - width) + x_start;
            //     println!("(x_start, x_end): {:?}", (x_start, x_end));
            // } else {
            //     let scale = view_size.0 / settings.resolution.0 as f32;
            //     let height = settings.resolution.1 as f32 / scale;
            //     y_start = (view_size.1 - height) / 2.0;
            //     y_end = height;
            // }

            Mat4::orthographic_rh_gl(-x_start, x_end, -y_start, y_end, -1.0, 1.0)
        }
    };

    uniforms.projection = projection;
    uniforms.model =
        crate::rendering::param_to_instance_transform(rotation, scale, offset, position);

    let color = color.to_percentage();

    uniforms.source = Vec4::new(source.x, source.y, source.width, source.height);
    uniforms.color = Vec4::new(color.0, color.1, color.2, color.3);

    if let Some(texture) = asset_store.get_texture(&texture_key) {
        texture.inner.set_filter(&mut ctx, texture.filter);
        ctx.apply_bindings(&texture.bindings);
        ctx.apply_uniforms(&uniforms);
        ctx.draw(0, 6, 1);
    }
}

#[inline]
pub fn ascii_character_list() -> Vec<char> {
    (0..255).filter_map(::std::char::from_u32).collect()
}

#[inline]
fn is_in_view(
    _sprite: &Sprite,
    _pos: &Position,
    _camera: &Camera,
    _screen_size: &(f32, f32),
) -> bool {
    true
}

#[inline]
fn get_camera_and_camera_position(world: &EmeraldWorld) -> (Camera, Position) {
    let mut cam = Camera::default();
    let mut cam_position = Position::new(0.0, 0.0);
    let mut entity_holding_camera: Option<Entity> = None;

    for (id, camera) in world.query::<&Camera>().iter() {
        if camera.is_active {
            cam = camera.clone();
            entity_holding_camera = Some(id);
        }
    }

    if let Some(entity) = entity_holding_camera {
        if let Ok(position) = world.get_mut::<Position>(entity) {
            cam_position = position.clone();
        }
    }

    (cam, cam_position)
}

#[derive(Clone)]
enum Drawable {
    Aseprite {
        sprite: Sprite,
        rotation: f32,
        color: Color,
        centered: bool,
        scale: Vector2<f32>,
        offset: Vector2<f32>,
        z_index: f32,
        visible: bool,
    },
    Sprite {
        sprite: Sprite,
    },
    ColorRect {
        color_rect: ColorRect,
    },
    Label {
        label: Label,
    },
}

#[derive(Clone)]
struct DrawCommand {
    pub drawable: Drawable,
    pub position: Position,
    pub z_index: f32,
}
