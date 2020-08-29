use crate::*;
use crate::world::*;
use crate::rendering::*;
use crate::rendering::components::*;
use crate::rendering::components::aseprite::types::*;
use crate::rendering::texture::{Texture};
use crate::rendering::font::FontKey;

use std::fs::File;
use std::io::Read as StdIoRead;
use std::rc::Rc;

use miniquad_text_fontdue as quad_text;

use miniquad::{
    BlendFactor, BlendState, BlendValue, Equation,
    Pipeline, PipelineParams,
    Bindings, BufferType, BufferLayout,
    Context, Buffer, VertexFormat,
    VertexAttribute, Shader};
use glam::{Vec2, Vec4, Mat4};
use std::collections::HashMap;

pub struct RenderingEngine {
    settings: RenderSettings,
    pipeline: Pipeline,
    textures: HashMap<TextureKey, Texture>,
    fonts: HashMap<FontKey, Rc<quad_text::FontTexture>>,
    text_system: quad_text::TextSystem,
    uid: usize,
    text_displays: HashMap<TextDisplayKey, quad_text::TextDisplay<Rc<quad_text::FontTexture>>>,
}
impl RenderingEngine {
    pub fn new(mut ctx: &mut Context, settings: RenderSettings) -> Self {
        let shader = Shader::new(ctx, VERTEX, FRAGMENT, META).unwrap();

        let mut params = PipelineParams::default();
        params.depth_write = true;
        params.color_blend = Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha))
        );
        params.alpha_blend = Some(BlendState::new(
            Equation::Add,
            BlendFactor::Zero,
            BlendFactor::One)
        );

        let pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("position", VertexFormat::Float2),
            ],
            shader,
            params,
        );


        let mut textures: HashMap<TextureKey, Texture> = HashMap::new();
        let fonts = HashMap::new();
        let text_system = quad_text::TextSystem::new(&mut ctx);

        let default_texture = Texture::default(&mut ctx).unwrap();
        textures.insert(TextureKey::default(), default_texture);

        RenderingEngine {
            settings,
            pipeline,
            textures,
            fonts,
            text_system,
            uid: 0,
            text_displays: HashMap::new(),
        }
    }

    pub fn update(&mut self, delta: f32, world: &mut hecs::World) {
        for (id, (aseprite)) in world.query::<&mut Aseprite>().iter() {
            aseprite.add_delta(delta);
        }
    }

    #[inline]
    pub fn draw_world(&mut self, mut ctx: &mut Context, world: &mut EmeraldWorld) {
        let screen_size = match self.settings.scalar {
            ScreenScalar::Keep => (self.settings.resolution.0 as f32, self.settings.resolution.1 as f32),
            ScreenScalar::None => ctx.screen_size(),
        };
        let camera = Camera::default(); // Get first active camera in world here, or default

        let mut sprites = Vec::with_capacity(100);

        for (id, (aseprite, position)) in world.inner.query::<(&mut Aseprite, &Position)>().iter() {
            aseprite.update();

            if is_in_view(&aseprite.sprite, &position, &camera, &screen_size) {
                sprites.push((aseprite.sprite.clone(), position.clone()));
            }
        }

        for (id, (sprite, position)) in world.inner.query::<(&Sprite, &Position)>().iter() {
            if is_in_view(&sprite, &position, &camera, &screen_size) {
                sprites.push((sprite.clone(), position.clone()));
            }
        }

        for (id, (color_rect, position)) in world.inner.query::<(&ColorRect, &Position)>().iter() {
            self.draw_color_rect(&mut ctx, &color_rect, &position);
        }

        sprites.sort_by(|a, b| a.0.z_index.partial_cmp(&b.0.z_index).unwrap() );

        for (sprite, position) in sprites.iter() {
            self.draw_sprite(&mut ctx, &sprite, &position);
        }

        ctx.end_render_pass();

        for (id, (mut label, position)) in world.inner.query::<(&mut Label, &Position)>().iter() {
            self.draw_label(&mut ctx, &mut label, &position);
        }
    }

    #[inline]
    pub fn draw_colliders(&mut self, mut ctx: &mut Context, world: &mut EmeraldWorld, collider_color: Color) {
        let mut color_rect = ColorRect::default();
        color_rect.color = collider_color;

        for (id, body_handle) in world.inner.query::<&RigidBodyHandle>().iter() {
            if let Some(body) = world.physics_engine.bodies.get(*body_handle) {
                for collider_handle in body.colliders() {
                    if let Some(collider) = world.physics_engine.colliders.get(collider_handle.clone()) {
                        let bf = &world.physics_engine.broad_phase;
                        let aabb = collider.compute_aabb();
                        let pos = Position::new(aabb.center().coords.x, aabb.center().coords.y);
                        color_rect.width = aabb.half_extents().x as u32 * 2;
                        color_rect.height = aabb.half_extents().y as u32 * 2;

                        self.draw_color_rect(&mut ctx, &color_rect, &pos);
                    }
                }
            }
        }
    }

    #[inline]
    pub(crate) fn begin(&mut self, ctx: &mut Context) {
        ctx.begin_default_pass(Default::default());
        ctx.clear(Some(self.settings.background_color.percentage()), None, None);
    }

    #[inline]
    pub(crate) fn render(&mut self, ctx: &mut Context) {
        ctx.end_render_pass();
        ctx.commit_frame();
    }

    #[inline]
    pub(crate) fn draw_color_rect(&mut self, mut ctx: &mut Context, color_rect: &ColorRect, position: &Position) {
        ctx.apply_pipeline(&self.pipeline);

        let (width, height) = (color_rect.width, color_rect.height);

        let mut offset = color_rect.offset.clone();

        if color_rect.centered {
            offset.x -= (color_rect.width / 2) as f32;
            offset.y -= (color_rect.height / 2) as f32;
        }

        let real_scale = Vec2::new(
            width as f32,
            height as f32,
        );
        let real_position = Vec2::new(
            position.x + offset.x,
            position.y + offset.y,
        );

        self.draw_texture(
            &mut ctx,
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
    pub(crate) fn draw_sprite(&mut self, mut ctx: &mut Context, sprite: &Sprite, position: &Position) {
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
                offset.x -= texture.width as f32 / 2.0;
                offset.y -= texture.height as f32 / 2.0;
            } else {
                offset.x -= sprite.target.width / 2.0;
                offset.y -= sprite.target.height / 2.0;
            }
        }
        

        let real_scale = Vec2::new(
            sprite.scale.x * target.width * (f32::from(texture.width)),
            sprite.scale.y * target.height * (f32::from(texture.height)),
        );
        let real_position = Vec2::new(
            position.x + offset.x,
            position.y + offset.y,
        );

        self.draw_texture(&mut ctx,
            &sprite.texture_key,
            sprite.z_index,
            real_scale,
            sprite.rotation,
            Vec2::new(0.0, 0.0),
            real_position,
            target,
            WHITE)
    }

    #[inline]
    fn draw_texture(&mut self,
        mut ctx: &mut Context,
        texture_key: &TextureKey,
        z_index: f32,
        scale: Vec2,
        rotation: f32,
        offset: Vec2,
        position: Vec2,
        source: Rectangle,
        color: Color,
    ) {
        let texture = self.textures.get(&texture_key).unwrap();
        let view_size = ctx.screen_size();
        let mut uniforms = Uniforms::default();

        let projection = match self.settings.scalar {
            ScreenScalar::Keep => Mat4::orthographic_rh_gl(0.0, self.settings.resolution.0 as f32, 0.0,self.settings.resolution.1 as f32, -1.0, 1.0),
            ScreenScalar::None => Mat4::orthographic_rh_gl(0.0, view_size.0, 0.0, view_size.1, -1.0, 1.0),
        };

        uniforms.projection = projection;
        uniforms.model = crate::rendering::param_to_instance_transform(
            rotation,
            scale,
            offset,
            position,
        );

        let color = color.percentage();

        uniforms.source = Vec4::new(source.x, source.y, source.width, source.height);
        uniforms.color = Vec4::new(color.0, color.1, color.2, color.3);
        // uniforms.z_index = z_index;
        
        texture.inner.set_filter(&mut ctx, texture.filter);

        ctx.apply_bindings(&texture.bindings);
        ctx.apply_uniforms(&uniforms);
        ctx.draw(0, 6, 1);
    }

    pub fn draw_label(&mut self, mut ctx: &mut Context, mut label: &mut Label, position: &Position) {
        let start = Instant::now();
        let (w, h) = ctx.screen_size();

        if let Some(mut text_display) = self.text_displays.get_mut(&label.text_display_key) {
            let text_width = text_display.get_width();

            if !label.is_text_up_to_date {
                text_display.set_text(&mut ctx, &label.text);
                label.is_text_up_to_date = true;
            }
    
            #[rustfmt::skip]
            // let matrix = crate::rendering::param_to_instance_transform(
            //     0.0,
            //     Vec2::new(1.0, 1.0),
            //     Vec2::new(0.0, 0.0),
            //     Vec2::new(position.x, position.y),
            // ).to_cols_array_2d();
            let matrix:[[f32; 4]; 4] = glam::Mat4::from_cols_array(&[
                2.0 / text_width, 0.0, 0.0, 0.0,
                0.0, 2.0 * (w as f32) / (h as f32) / text_width, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                -1.0, -1.0, 0.0, 1.0f32,
            ]).to_cols_array_2d();
    
            quad_text::draw(ctx, &text_display, &self.text_system, matrix, (0.0, 0.0, 0.0, 1.0));
            let end = Instant::now();
        }
    }

    #[inline]
    pub fn aseprite_with_animations<T: Into<String>>(&mut self,
            mut ctx: &mut Context,
            texture_file: File,
            texture_file_path: T,
            animation_file: File,
            _animation_file_path: T
        ) -> Result<Aseprite, EmeraldError> {
        let sprite = self.sprite(&mut ctx, texture_file, texture_file_path)?;

        Aseprite::new(sprite, animation_file)
    }

    #[inline]
    pub fn sprite<T: Into<String>>(&mut self, mut ctx: &mut Context, file: File, path: T) -> Result<Sprite, EmeraldError> {
        let key = self.texture(&mut ctx, file, path)?;

        Ok(Sprite::from_texture(key))
    }

    #[inline]
    pub fn label<T: Into<String>>(&mut self, mut ctx: &mut Context, text: T, font_key: FontKey) -> Result<Label, EmeraldError> {
        if let Some(font) = self.fonts.get(&font_key) {
            let mut display = quad_text::TextDisplay::new(&mut ctx, &self.text_system, font.clone(), &text.into());
            let key = TextDisplayKey(self.uid);
            self.text_displays.insert(key.clone(), display);
            let mut label = Label::new(key);

            return Ok(label);
        }


        Err(EmeraldError::new(format!("Unable to get font with {:?}", font_key)))
    }

    #[inline]
    pub fn texture<T: Into<String>>(&mut self, mut ctx: &mut Context, file: File, path: T) -> Result<TextureKey, EmeraldError> {
        let path: String = path.into();
        let key = TextureKey::new(path.clone());

        if !self.textures.contains_key(&key) {
            let texture = Texture::new(&mut ctx, file)?;
            self.textures.insert(key.clone(), texture);
        }

        Ok(key)
    }

    pub fn pack_texture(&mut self, mut ctx: &mut Context, name: &str, bytes: Vec<u8>) {
        let texture = Texture::from_png_bytes(&mut ctx, bytes.as_slice()).unwrap();
        let key = TextureKey::new(name.to_string());
        
        self.textures.insert(key, texture);
    }

    #[inline]
    pub fn font<T: Into<String>>(&mut self, mut ctx: &mut Context, mut font_data: Vec<u8>, path: T, font_size: u32) -> Result<FontKey, EmeraldError> {
        let path: String = path.into();
        let key = FontKey::new(&path, font_size);

        if self.fonts.contains_key(&key) {
            return Ok(key);
        }

        let font = quad_text::FontTexture::new(
            &mut ctx,
            font_data.as_slice(),
            font_size,
            quad_text::FontAtlas::ascii_character_list(),
        ).unwrap();

        self.fonts.insert(key.clone(), Rc::new(font));

        self.uid += 1;

        Ok(key)
    }
}

#[inline]
fn is_in_view(sprite: &Sprite, pos: &Position, camera: &Camera, screen_size: &(f32, f32)) -> bool {
    true
}
