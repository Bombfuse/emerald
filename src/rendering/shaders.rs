use miniquad::*;
use glam::{Mat4, Vec2, Vec4};

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
}
impl Default for Uniforms {
    fn default() -> Uniforms {
        Uniforms {
            projection: Mat4::orthographic_lh(0.0, 1.0, 0.0, 1.0, -1.0, 1.0),
            model: Mat4::identity(),
            source: Vec4::new(0.0, 0.0, 1.0, 1.0),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

pub const VERTEX: &str = r#"
#version 100

attribute vec2 position;

varying lowp vec4 color;
varying lowp vec2 uv;

uniform mat4 Projection;
uniform vec4 Source;
uniform vec4 Color;
uniform mat4 Model;
uniform float depth;

void main() {
    gl_Position = Projection * Model * vec4(position, 0, 1);
    gl_Position.z = depth;
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

pub const META: ShaderMeta = ShaderMeta {
    images: &["tex"],
    uniforms: UniformBlockLayout {
        uniforms: &[
            UniformDesc::new("Projection", UniformType::Mat4),
            UniformDesc::new("Model", UniformType::Mat4),
            UniformDesc::new("Source", UniformType::Float4),
            UniformDesc::new("Color", UniformType::Float4),
        ],
    },
};

// Credit(https://github.com/not-fl3/good-web-game/blob/master/src/graphics/image.rs#L129)
pub(crate) fn param_to_instance_transform(rotation: f32, scale: Vec2, offset: Vec2, dest: Vec2) -> Mat4 {
    let cosr = rotation.cos();
    let sinr = rotation.sin();
    let m00 = cosr * scale.x();
    let m01 = -sinr * scale.y();
    let m10 = sinr * scale.x();
    let m11 = cosr * scale.y();
    let m03 = offset.x() * (1.0 - m00) - offset.y() * m01 + dest.x();
    let m13 = offset.y() * (1.0 - m11) - offset.x() * m10 + dest.y();

    Mat4::from_cols(
        Vec4::new(m00, m10, 0.0, 0.0),
        Vec4::new(m01, m11, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
        Vec4::new(m03, m13, 0.0, 1.0),
    )
}