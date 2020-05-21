
use miniquad::*;

#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}

#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

#[repr(C)]
pub struct Uniforms {
    pub offset: (f32, f32),
}

pub const VERTEX: &str = r#"#version 100
attribute vec2 pos;
attribute vec2 uv;
uniform vec2 offset;
varying lowp vec2 texcoord;
void main() {
    gl_Position = vec4(pos + offset, 0, 1);
    texcoord = uv;
}"#;

pub const FRAGMENT: &str = r#"#version 100
varying lowp vec2 texcoord;
uniform sampler2D tex;
void main() {
    gl_FragColor = texture2D(tex, texcoord);
}"#;

pub const META: ShaderMeta = ShaderMeta {
    images: &["tex"],
    uniforms: UniformBlockLayout {
        uniforms: &[UniformDesc::new("offset", UniformType::Float2)],
    },
};
