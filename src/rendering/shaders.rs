use miniquad::*;
use glam::{Mat4, Vec2, Vec4};

#[repr(C)]
pub struct Vertex {
    pub pos: Vec2,
    pub uv: Vec2,
}

#[repr(C)]
pub struct Uniforms {
    pub model: Mat4,
    pub target: Vec4,
    pub offset: Vec2,
    pub view_size: Vec2,
    pub z_index: f32,
}

pub const VERTEX: &str = r#"
#version 100

attribute vec2 pos;
attribute vec2 uv;

uniform vec2 offset;
uniform vec2 view_size;
uniform float z_index;
uniform mat4 model;
uniform vec4 target;

varying lowp vec2 texcoord;
varying lowp vec4 color;

void main() {
    gl_Position = model * vec4(2.0 * (pos.x + offset.x) / view_size.x - 1.0, 1.0 - 2.0 * (pos.y + offset.y) / view_size.y, 0, 1);
    gl_Position.z = z_index;

    texcoord.x = uv.x + target.x;
    texcoord.y = uv.y + target.y;

    color = vec4(1.0, 1.0, 1.0, 1.0);
}"#;

pub const FRAGMENT: &str = r#"
#version 100

varying lowp vec2 texcoord;
varying lowp vec4 color;

uniform sampler2D tex;

void main() {
    gl_FragColor = texture2D(tex, texcoord) * color;

    if (color.a <= 0.0) {
        discard;
    }
}"#;

pub const META: ShaderMeta = ShaderMeta {
    images: &["tex"],
    uniforms: UniformBlockLayout {
        uniforms: &[
            UniformDesc::new("model", UniformType::Mat4),
            UniformDesc::new("target", UniformType::Float4),
            UniformDesc::new("offset", UniformType::Float2),
            UniformDesc::new("view_size", UniformType::Float2),
            UniformDesc::new("z_index", UniformType::Float1),
        ],
    },
};
