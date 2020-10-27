use crate::*;
use crate::rendering::shaders::*;
use crate::rendering::font::*;
use miniquad::{Bindings, Buffer, BufferType, Context, FilterMode};

use glam::Vec2;

#[derive(Clone, Debug)]
pub struct Texture {
    pub(crate) inner: miniquad::Texture,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) filter: FilterMode,
    pub(crate) bindings: Bindings,
}
impl Texture {
    pub(crate) fn new(mut ctx: &mut Context, data: Vec<u8>) -> Result<Self, EmeraldError> {
        Self::from_png_bytes(&mut ctx, &data)
    }

    pub fn default(mut ctx: &mut Context) -> Result<Self, EmeraldError> {
        let pixels: [u8; 4 * 4 * 4] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let texture = miniquad::Texture::from_rgba8(ctx, 4, 4, &pixels);

        Self::from_texture(&mut ctx, texture)
    }

    pub fn from_png_bytes(ctx: &mut Context, bytes: &[u8]) -> Result<Self, EmeraldError> {
        let img = image::load_from_memory(&bytes)
            .unwrap_or_else(|e| panic!(e))
            .to_rgba();
        let img = image::imageops::flip_vertical(&img);

        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        Self::from_rgba8(ctx, width, height, &bytes)
    }

    pub(crate) fn from_rgba8(
        mut ctx: &mut Context,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> Result<Self, EmeraldError> {
        let texture = miniquad::Texture::from_rgba8(&mut ctx, width, height, bytes);

        Self::from_texture(&mut ctx, texture)
    }

    pub(crate) fn from_texture(
        ctx: &mut miniquad::Context,
        texture: miniquad::Texture,
    ) -> Result<Self, EmeraldError> {
        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { position: Vec2::new(0.0, 0.0) },
            Vertex { position: Vec2::new(1.0, 0.0) },
            Vertex { position: Vec2::new(1.0, 1.0) },
            Vertex { position: Vec2::new(0.0, 1.0) },
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
            filter: FilterMode::Nearest,
        })
    }

    pub(crate) fn update(&mut self, ctx: &mut miniquad::Context, font_image: &FontImage) {
        assert_eq!(self.inner.width, font_image.width as u32);
        assert_eq!(self.inner.height, font_image.height as u32);

        self.inner.update(ctx, &font_image.bytes);
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
