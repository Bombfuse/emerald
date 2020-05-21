use crate::*;

pub struct ColorRect {
    pub color: Color,
    pub position: Vector2<f32>,
    pub width: u32,
    pub height: u32,
    pub centered: bool,
}