use crate::rendering::font::*;
use crate::rendering::shaders::*;
use crate::*;
use glam::Vec2;
use miniquad::{Bindings, Buffer, BufferType, Context, FilterMode};
use std::sync::Arc;

pub const EMERALD_DEFAULT_TEXTURE_NAME: &str = "emerald_default_texture";

#[derive(Clone, Debug)]
pub struct Texture {
    pub(crate) key: TextureKey,
    pub(crate) inner: miniquad::Texture,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) filter: FilterMode,
    pub(crate) bindings: Bindings,
}
impl Texture {
    pub(crate) fn new(
        mut ctx: &mut Context,
        key: TextureKey,
        data: Vec<u8>,
    ) -> Result<Self, EmeraldError> {
        Self::from_png_bytes(&mut ctx, key, &data)
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

        Self::from_texture(
            &mut ctx,
            TextureKey::new(EMERALD_DEFAULT_TEXTURE_NAME),
            texture,
        )
    }

    pub fn from_png_bytes(
        ctx: &mut Context,
        key: TextureKey,
        bytes: &[u8],
    ) -> Result<Self, EmeraldError> {
        let img = image::load_from_memory(bytes)?.to_rgba8();
        let img = image::imageops::flip_vertical(&img);

        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        Self::from_rgba8(ctx, key, width, height, &bytes)
    }

    pub(crate) fn from_rgba8(
        mut ctx: &mut Context,
        key: TextureKey,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> Result<Self, EmeraldError> {
        let texture = miniquad::Texture::from_rgba8(&mut ctx, width, height, bytes);

        Self::from_texture(&mut ctx, key, texture)
    }

    pub(crate) fn from_texture(
        ctx: &mut miniquad::Context,
        key: TextureKey,
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
            index_buffer,
            images: vec![texture],
        };

        Ok(Texture {
            key,
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
pub struct TextureKey(pub(crate) Arc<String>);
impl TextureKey {
    pub(crate) fn new<T: Into<String>>(texture_path: T) -> Self {
        TextureKey(Arc::new(texture_path.into()))
    }

    pub fn get_name(&self) -> String {
        self.0.as_ref().clone()
    }
}
impl Default for TextureKey {
    fn default() -> TextureKey {
        TextureKey(Arc::new(String::from(EMERALD_DEFAULT_TEXTURE_NAME)))
    }
}
