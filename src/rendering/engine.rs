use crate::rendering::components::*;
use crate::rendering::texture::Texture;
use crate::rendering::*;
use crate::world::*;
use crate::*;

use glam::{Mat4, Vec2, Vec4};
use miniquad::*;
use std::collections::HashMap;

pub struct RenderingEngine {
    pub(crate) settings: RenderSettings,
    pipeline: Pipeline,
    pub(crate) textures: HashMap<TextureKey, Texture>,
    pub(crate) fonts: HashMap<FontKey, Font>,
    render_target: Option<miniquad::Texture>,
}
impl RenderingEngine {
    pub fn new(mut ctx: &mut Context, settings: RenderSettings) -> Self {
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

        let mut textures = HashMap::new();
        let default_texture = Texture::default(&mut ctx).unwrap();
        textures.insert(TextureKey::default(), default_texture);

        let fonts = HashMap::new();

        RenderingEngine {
            settings,
            pipeline,
            textures,
            fonts,
            render_target: None,
        }
    }

    pub(crate) fn update(&mut self, delta: f64, world: &mut hecs::World) {
        for (_id, aseprite) in world.query::<&mut Aseprite>().iter() {
            aseprite.add_delta(delta as f32);
        }
    }

    #[inline]
    pub fn draw_world(&mut self, mut ctx: &mut Context, world: &mut EmeraldWorld) -> Result<(), EmeraldError> {
        let screen_size = self.get_screen_size(ctx);
        let (camera, camera_position) = get_camera_and_camera_position(world);

        let mut draw_queue = Vec::new();

        for (_id, (aseprite, position)) in world.inner.query::<(&mut Aseprite, &Position)>().iter()
        {
            aseprite.update();

            if is_in_view(&aseprite.sprite, &position, &camera, &screen_size) {
                let drawable = Drawable::Sprite {
                    sprite: aseprite.sprite.clone(),
                };

                draw_queue.push(DrawCommand {
                    drawable,
                    position: position.clone(),
                    z_index: aseprite.sprite.z_index,
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

                pos
            };

            match draw_command.drawable {
                Drawable::Sprite { sprite } => self.draw_sprite(&mut ctx, &sprite, &position),
                Drawable::ColorRect { color_rect } => {
                    self.draw_color_rect(&mut ctx, &color_rect, &position)
                }
                Drawable::Label { label } => self.draw_label(&mut ctx, &label, &position)?,
            }
        }

        ctx.end_render_pass();

        Ok(())
    }

    #[inline]
    pub fn draw_colliders(
        &mut self,
        mut ctx: &mut Context,
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

                        self.draw_color_rect(&mut ctx, &color_rect, &position);
                    }
                }
            }
        }
    }

    #[inline]
    pub(crate) fn begin(&mut self, ctx: &mut Context) {
        ctx.begin_default_pass(Default::default());
        ctx.clear(
            Some(self.settings.background_color.to_percentage()),
            None,
            None,
        );
    }

    /// Begins the process of rendering to a texture the size of the screen.
    #[inline]
    pub(crate) fn begin_texture(&mut self, ctx: &mut Context) {
        ctx.clear(
            Some(self.settings.background_color.to_percentage()),
            None,
            None,
        );
        let (w, h) = ctx.screen_size();

        self.render_target = Some(miniquad::Texture::new_render_texture(
            ctx,
            TextureParams {
                width: w as _,
                height: h as _,
                format: TextureFormat::Depth,
                ..Default::default()
            },
        ));
    }

    #[inline]
    pub(crate) fn render(&mut self, ctx: &mut Context) {
        ctx.end_render_pass();
        ctx.commit_frame();
    }

    pub(crate) fn render_to_texture(
        &mut self,
        mut ctx: &mut Context,
    ) -> Result<Texture, EmeraldError> {
        if let Some(texture) = self.render_target {
            let texture = Texture::from_texture(&mut ctx, texture)?;
            return Ok(texture);
        }

        Err(EmeraldError::new(
            "No texture found. Did you begin this rendering pass with 'begin_texture()`?",
        ))
    }

    pub(crate) fn draw_label(&mut self, mut ctx: &mut Context, label: &Label, position: &Position) -> Result<(), EmeraldError> {
        if let Some(font) = self.fonts.get_mut(&label.font_key) {
            ctx.apply_pipeline(&self.pipeline);

            let mut draw_calls: Vec<(
                f32, // z_index
                Vec2, // real_scale
                Vec2, // real_position
                Rectangle, // target
                Color, // color
            )> = Vec::new();

            let mut total_width = 0.0;
            for character in label.text.chars() {
                if !font.characters.contains_key(&(character, label.font_size)) {
                    font.cache_glyph(&mut ctx, character, label.font_size)?;
                }

                let font_data = &font.characters[&(character, label.font_size)];
                {
                    let left_coord = font_data.offset_x as f32 * label.scale + total_width;
                    let top_coord = label.font_size as f32 * label.scale
                        - font_data.glyph_h as f32 * label.scale
                        - font_data.offset_y as f32 * label.scale;

                    total_width += font_data.advance * label.scale;

                    let target = Rectangle::new(
                        (font_data.glyph_x as f32) / font.font_texture.width as f32,
                        (font_data.glyph_y as f32) / font.font_texture.height as f32,
                        (font_data.glyph_w as f32) / font.font_texture.width as f32,
                        (font_data.glyph_h as f32) / font.font_texture.height as f32,
                    );

                    let real_scale = Vec2::new(
                        label.scale * target.width * font.font_texture.width as f32,
                        label.scale * target.height * font.font_texture.height as f32 * -1.0,
                    );
                    let real_position = Vec2::new(position.x + label.offset.x + left_coord, position.y - label.offset.y - top_coord);

                    draw_calls.push((
                        label.z_index,
                        real_scale,
                        real_position,
                        target,
                        label.color,
                    ));
                }
            }

            for draw_call in draw_calls {
                let (z_index, real_scale, mut real_position, target, mut color) = draw_call;

                if label.centered {
                    real_position.x -= total_width / 2.0;
                }

                if !label.visible {
                    color.a = 0;
                }

                draw_texture(
                    &self.settings,
                    &mut ctx,
                    &font.font_texture,
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
        let texture = self.textures.get(&TextureKey::default()).unwrap();

        draw_texture(
            &self.settings,
            &mut ctx,
            texture,
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
    pub(crate) fn draw_sprite(
        &mut self,
        mut ctx: &mut Context,
        sprite: &Sprite,
        position: &Position,
    ) {
        if !sprite.visible {
            return;
        }

        ctx.apply_pipeline(&self.pipeline);

        let texture = self.textures.get(&sprite.texture_key).unwrap();
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
        let texture = self.textures.get(&sprite.texture_key).unwrap();

        draw_texture(
            &self.settings,
            &mut ctx,
            texture,
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
    pub fn aseprite_with_animations<T: Into<String>>(
        &mut self,
        mut ctx: &mut Context,
        texture_data: Vec<u8>,
        texture_file_path: T,
        animation_data: Vec<u8>,
        _animation_file_path: T,
    ) -> Result<Aseprite, EmeraldError> {
        let sprite = self.sprite_from_data(&mut ctx, texture_data, texture_file_path)?;

        Aseprite::new(sprite, animation_data)
    }

    #[inline]
    pub fn sprite<T: Into<String>>(&mut self, path: T) -> Result<Sprite, EmeraldError> {
        let key = self.texture(path)?;

        Ok(Sprite::from_texture(key))
    }

    #[inline]
    pub fn sprite_from_data<T: Into<String>>(
        &mut self,
        mut ctx: &mut Context,
        data: Vec<u8>,
        path: T,
    ) -> Result<Sprite, EmeraldError> {
        let key = self.texture_from_data(&mut ctx, data, path)?;

        Ok(Sprite::from_texture(key))
    }

    #[inline]
    pub fn texture<T: Into<String>>(&mut self, path: T) -> Result<TextureKey, EmeraldError> {
        let path: String = path.into();
        let key = TextureKey::new(path.clone());

        if !self.textures.contains_key(&key) {
            return Err(EmeraldError::new(format!(
                "Unable to get texture for {}",
                path
            )));
        }

        Ok(key)
    }

    #[inline]
    pub fn font<T: Into<String>>(
        &mut self,
        ctx: &mut Context,
        path: T,
        font_data: Vec<u8>,
        font_size: u32,
    ) -> Result<FontKey, EmeraldError> {
        let font_path: String = path.into();
        let key = FontKey::new(font_path.clone(), font_size);
        let font = Font::from_bytes(ctx, font_data.as_slice(), font_size)?;
        self.fonts.insert(key.clone(), font);
        self.populate_font_cache(ctx, &key, &ascii_character_list(), 24)?;

        Ok(key)
    }

    #[inline]
    pub fn texture_from_data<T: Into<String>>(
        &mut self,
        mut ctx: &mut Context,
        data: Vec<u8>,
        path: T,
    ) -> Result<TextureKey, EmeraldError> {
        let path: String = path.into();
        let key = TextureKey::new(path.clone());

        if !self.textures.contains_key(&key) {
            let texture = Texture::new(&mut ctx, data)?;
            self.textures.insert(key.clone(), texture);
        }

        Ok(key)
    }

    pub fn pack_texture(
        &mut self,
        mut ctx: &mut Context,
        name: &str,
        bytes: Vec<u8>,
    ) -> Result<(), EmeraldError> {
        let texture = Texture::from_png_bytes(&mut ctx, bytes.as_slice())?;
        let key = TextureKey::new(name.to_string());

        self.textures.insert(key, texture);

        Ok(())
    }

    #[inline]
    pub fn make_active_camera(
        &mut self,
        entity: Entity,
        world: &mut EmeraldWorld,
    ) -> Result<(), EmeraldError> {
        let mut set_camera = false;
        if let Ok(mut camera) = world.get_mut::<Camera>(entity.clone()) {
            camera.is_active = true;
            set_camera = true;
        }

        if set_camera {
            for (id, mut camera_to_disable) in world.query::<&mut Camera>().iter() {
                if id != entity {
                    camera_to_disable.is_active = false;
                }
            }
        }

        Err(EmeraldError::new(format!(
            "Entity {:?} either does not exist or does not hold a camera",
            entity
        )))
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

    #[inline]
    pub fn populate_font_cache(
        &mut self,
        ctx: &mut Context,
        key: &FontKey,
        characters: &[char],
        size: u16,
    ) -> Result<(), EmeraldError> {
        if let Some(font) = self.fonts.get_mut(key) {
            for character in characters {
                font.cache_glyph(ctx, *character, size)?;
            }
        }

        Ok(())
    }
}

#[inline]
fn draw_texture(
    settings: &RenderSettings,
    mut ctx: &mut Context,
    texture: &Texture,
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
        ScreenScalar::Keep => Mat4::orthographic_rh_gl(
            0.0,
            settings.resolution.0 as f32,
            0.0,
            settings.resolution.1 as f32,
            -1.0,
            1.0,
        ),
    };

    uniforms.projection = projection;
    uniforms.model =
        crate::rendering::param_to_instance_transform(rotation, scale, offset, position);

    let color = color.to_percentage();

    uniforms.source = Vec4::new(source.x, source.y, source.width, source.height);
    uniforms.color = Vec4::new(color.0, color.1, color.2, color.3);
    texture.inner.set_filter(&mut ctx, texture.filter);

    ctx.apply_bindings(&texture.bindings);
    ctx.apply_uniforms(&uniforms);
    ctx.draw(0, 6, 1);
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

#[derive(Clone, Debug)]
enum Drawable {
    Sprite { sprite: Sprite },
    ColorRect { color_rect: ColorRect },
    Label { label: Label },
}

#[derive(Clone, Debug)]
struct DrawCommand {
    pub drawable: Drawable,
    pub position: Position,
    pub z_index: f32,
}
