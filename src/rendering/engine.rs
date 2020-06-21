use crate::*;
use crate::world::*;
use crate::rendering::*;
use crate::rendering::texture::{Texture};
use crate::rendering::font::FontKey;

use std::fs::File;
use std::io::Read as StdIoRead;

use miniquad::{Pipeline, Bindings, BufferType, BufferLayout, Context, Buffer, VertexFormat, VertexAttribute, Shader};
use legion::prelude::{Schedulable, Query, SystemBuilder, Read, Write, IntoQuery};
use std::collections::HashMap;
use fontdue::{Font, FontSettings};

pub struct RenderingEngine {
    settings: RenderSettings,
    pipeline: Pipeline,
    textures: HashMap<TextureKey, Texture>,
    fonts: HashMap<FontKey, Font>,
}
impl RenderingEngine {
    pub fn new(mut ctx: &mut Context, settings: RenderSettings) -> Self {
        let shader = Shader::new(ctx, VERTEX, FRAGMENT, META).unwrap();

        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
        );

        let mut textures: HashMap<TextureKey, Texture> = HashMap::new();
        let fonts: HashMap<FontKey, Font> = HashMap::new();

        let default_texture = Texture::default(&mut ctx).unwrap();
        textures.insert(TextureKey::default(), default_texture);

        RenderingEngine {
            settings,
            pipeline,
            textures,
            fonts,
        }
    }

    pub fn update(&mut self, mut ctx: &mut Context, world: &mut World) {
        let sprite_query = <(Read<Sprite>, Read<Position>)>::query();

        ctx.clear(Some(self.settings.background_color.percentage()), None, None);

        ctx.begin_default_pass(Default::default());
        
        ctx.apply_pipeline(&self.pipeline);

        for (sprite, position) in sprite_query.iter(&mut world.inner) {
            self.render_sprite(&mut ctx, &sprite, &position);
        }
        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn render_color_rect(&mut self, ctx: &mut Context, color_rect: &ColorRect) {}

    fn render_sprite(&mut self, ctx: &mut Context, sprite: &Sprite, position: &Position) {
        let texture = self.textures.get(&sprite.texture_key).unwrap();

        ctx.apply_bindings(&texture.bindings);
        ctx.apply_uniforms(&Uniforms {
            offset: (position.x, position.y),
            viewSize: (800.0, 600.0),
        });
        ctx.draw(0, 6, 1);
    }

    fn render_label(&mut self, ctx: &mut Context, label: &Label, position: &Position) {
        // Get font texture here
        // Render texture font at target characters in sequence
    }

    pub fn sprite(&mut self, mut ctx: &mut Context, path: &str) -> Result<Sprite, EmeraldError> {
        let key = TextureKey::new(path);

        if !self.textures.contains_key(&key) {
            let texture = Texture::new(&mut ctx, path)?;
            self.textures.insert(key.clone(), texture);
        }
        
        Ok(Sprite::from_texture(key))
    }

    pub fn font(&mut self, mut ctx: &mut Context, path: &str, _font_size: u16) -> Result<FontKey, EmeraldError> {
        let key = FontKey::new(path);

        if self.fonts.contains_key(&key) {
            return Ok(key);
        }

        let mut file = File::open(path)?;
        let mut font_data = Vec::new();
        file.read_to_end(&mut font_data)?;

        let font = Font::from_bytes(font_data.as_slice(), FontSettings::default())?;
        self.fonts.insert(key.clone(), font);

        // Create texture here big enough for fuckin regular letters shit or something idk man
        // Characters are hard
        // Just do the 0..26 for now
        // Just load texture to the engine textures, then point at it
        let size: u16 = 128;
        let mut bytes = Vec::with_capacity((size * size) as usize);

        for i in 0..(size * size) {
            bytes.push(0xFF);
        }

        let font_texture = Texture::from_rgba8(&mut ctx, size, size, &bytes)?;
        let texture_key = TextureKey::new(path);
        self.textures.insert(texture_key, font_texture);

        Ok(key)
    }
}