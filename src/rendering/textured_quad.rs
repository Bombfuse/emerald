use crate::*;
use crate::{rendering::font::*, texture_meta::TextureMeta};
use glam::Vec2;
use miniquad::{Bindings, Buffer, BufferType, Context, FilterMode};
use std::sync::Arc;

pub const EMERALD_DEFAULT_TEXTURE_NAME: &str = "emerald_default_texture";

#[derive(Clone, Debug)]
pub struct TexturedQuad {
    pub(crate) meta: TextureMeta,
    pub(crate) bindings: Bindings,
}
impl TexturedQuad {
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

        Ok(Self {
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
pub struct TextureKey {
    pub(crate) texture_path: Arc<String>,
    pub(crate) meta: TextureMeta,
}
impl TextureKey {
    pub(crate) fn new<T: Into<String>>(texture_path: T, meta: TextureMeta) -> Self {
        Self {
            texture_path: texture_path.into(),
            meta,
        }
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

use glam::{Mat4, Vec4};

#[repr(C)]
pub struct Vertex {
    pub position: Vec2,
}

#[repr(C)]
#[derive(Debug)]
pub struct Uniforms {
    pub projection: Mat4,
    pub model: Mat4,
    pub source: Vec4,
    pub color: Vec4,
    pub z_index: f32,
}
impl Default for Uniforms {
    fn default() -> Uniforms {
        Uniforms {
            projection: Mat4::orthographic_lh(0.0, 1.0, 0.0, 1.0, -100.0, 100.0),
            model: Mat4::IDENTITY,
            source: Vec4::new(0.0, 0.0, 1.0, 1.0),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            z_index: 0.0,
        }
    }
}

pub mod shaders {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"
    #version 100
    
    attribute vec2 position;
    
    varying lowp vec4 color;
    varying lowp vec2 uv;
    
    uniform mat4 Projection;
    uniform vec4 Source;
    uniform vec4 Color;
    uniform mat4 Model;
    
    uniform float z_index;
    
    void main() {
        gl_Position = Projection * Model * vec4(position, 0, 1);
        gl_Position.z = z_index;
        color = Color;
        uv = position * Source.zw + Source.xy;
    }"#;

    pub const FRAGMENT: &str = r#"
    #version 100
    
    varying lowp vec4 color;
    varying lowp vec2 uv;
    
    uniform sampler2D tex;
    
    void main() {
        gl_FragColor = texture2D(tex, uv) * color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![String::from("tex")],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Projection", UniformType::Mat4),
                    UniformDesc::new("Model", UniformType::Mat4),
                    UniformDesc::new("Source", UniformType::Float4),
                    UniformDesc::new("Color", UniformType::Float4),
                    UniformDesc::new("z_index", UniformType::Float1),
                ],
            },
        }
    }
}

// Credit(https://github.com/not-fl3/good-web-game/blob/master/src/graphics/image.rs#L129)
pub(crate) fn param_to_instance_transform(
    rotation: f32,
    scale: Vec2,
    offset: Vec2,
    dest: Vec2,
) -> Mat4 {
    let cosr = rotation.cos();
    let sinr = rotation.sin();
    let m00 = cosr * scale.x;
    let m01 = -sinr * scale.y;
    let m10 = sinr * scale.x;
    let m11 = cosr * scale.y;
    let m03 = offset.x * (1.0 - m00) - offset.y * m01 + dest.x;
    let m13 = offset.y * (1.0 - m11) - offset.x * m10 + dest.y;

    Mat4::from_cols(
        Vec4::new(m00, m10, 0.0, 0.0),
        Vec4::new(m01, m11, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
        Vec4::new(m03, m13, 0.0, 1.0),
    )
}
