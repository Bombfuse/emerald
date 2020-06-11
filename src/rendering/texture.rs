use crate::*;
use crate::rendering::shaders::*;
use miniquad::{Bindings, FilterMode, Context, BufferType, Buffer};

use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

#[derive(Clone, Debug)]
pub struct Texture {
    pub(crate) inner: miniquad::Texture,
    pub(crate) width: u16,
    pub(crate) height: u16,
    filter: FilterMode,
    pub(crate) bindings: Bindings,
}
impl Texture {
    pub fn new<P: AsRef<Path>>(mut ctx: &mut Context, path: P) -> Result<Self, EmeraldError> {
        let mut file = File::open(path)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;

        Self::from_png_bytes(&mut ctx, &bytes)
    }

    pub fn default(mut ctx: &mut Context) -> Result<Self, EmeraldError> {
        let pixels: [u8; 4 * 4 * 4] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
            0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let texture = miniquad::Texture::from_rgba8(ctx, 4, 4, &pixels);
        
        Self::from_texture(&mut ctx, texture)
    }

    pub fn from_png_bytes(ctx: &mut Context, bytes: &[u8]) -> Result<Self, EmeraldError> {
        let img = image::load_from_memory(&bytes)
            .unwrap_or_else(|e| panic!(e))
            .to_rgba();
        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        Self::from_rgba8(ctx, width, height, &bytes)
    }
    
    pub fn from_rgba8(
        mut ctx: &mut Context,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> Result<Self, EmeraldError> {
        let texture = miniquad::Texture::from_rgba8(&mut ctx, width, height, bytes);

        Self::from_texture(&mut ctx, texture)
    }

    pub fn from_texture(ctx: &mut miniquad::Context, texture: miniquad::Texture) -> Result<Self, EmeraldError> {
        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: 0.0, y: 0.0 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x: texture.width as f32, y: 0.0 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x: texture.width as f32, y: texture.height as f32 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: 0.0, y: texture.height as f32 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![texture],
        };

        Ok(Texture {
            width: texture.width as u16,
            height: texture.height as u16,
            inner: texture,
            bindings,
            filter: FilterMode::Linear,
        })
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct TextureKey(String);
impl TextureKey {
    pub fn new<T: Into<String>>(texture_path: T) -> Self {
        TextureKey(texture_path.into())
    }
}
impl Default for TextureKey {
    fn default() -> TextureKey {
        TextureKey(String::from("emerald_default_texture"))
    }
}