use crate::*;
use crate::world::*;
use crate::rendering::*;
use crate::rendering::texture::{Texture};

use paintbrush::*;
use miniquad::{Pipeline, Bindings, BufferType, BufferLayout, Context, Buffer, VertexFormat, VertexAttribute, Shader};
use legion::prelude::{Schedulable, Query, SystemBuilder, Read, Write, IntoQuery};
use std::collections::HashMap;

pub struct RenderingEngine {
    settings: RenderSettings,
    pipeline: Pipeline,
    textures: HashMap<TextureKey, Texture>,
}
impl RenderingEngine {
    pub fn new(mut ctx: &mut Context, settings: RenderSettings) -> Self {
        let shader = Shader::new(ctx, VERTEX, FRAGMENT, META);

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

        let default_texture = Texture::default(&mut ctx).unwrap();
        textures.insert(TextureKey::default(), default_texture);

        RenderingEngine {
            settings,
            pipeline,
            textures,
        }
    }

    pub fn update(&mut self, mut ctx: &mut Context, mut world: &mut World) {
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
            offset: (position.x, position.y)
        });
        ctx.draw(0, 6, 1);
    }

    pub fn sprite(&mut self, mut ctx: &mut Context, path: &str) -> Result<Sprite, EmeraldError> {
        // let key = String::from(path);

        // if !self.textures.contains_key(&key) {
        //     let texture = Texture::new(&mut ctx, path)?;
        //     self.textures.insert(key.clone(), texture);
        // }
        
        // let mut texture = self.textures.get_mut(&key).unwrap();


        Ok(Sprite::default())
    }
}