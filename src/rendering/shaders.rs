
use miniquad::*;

#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct Vertex {
    pub pos: Vec2,
    pub uv: Vec2,
}

#[repr(C)]
pub struct Uniforms {
    pub offset: (f32, f32),
    pub viewSize: (f32, f32),
}

pub const VERTEX: &str = r#"
#version 100

attribute vec2 pos;
attribute vec2 uv;

uniform vec2 offset;
uniform vec2 viewSize;

varying lowp vec2 texcoord;

void main() {
    gl_Position = vec4(2.0 * (pos.x + offset.x) / viewSize.x - 1.0, 1.0 - 2.0 * (pos.y + offset.y) / viewSize.y, 0, 1);
    texcoord = uv;
}"#;

pub const FRAGMENT: &str = r#"
#version 100

varying lowp vec2 texcoord;
uniform sampler2D tex;
void main() {
    gl_FragColor = texture2D(tex, texcoord);
}"#;

pub const META: ShaderMeta = ShaderMeta {
    images: &["tex"],
    uniforms: UniformBlockLayout {
        uniforms: &[
            UniformDesc::new("offset", UniformType::Float2),
            UniformDesc::new("viewSize", UniformType::Float2),
        ],
    },
};
