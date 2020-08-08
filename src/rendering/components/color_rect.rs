use crate::*;

#[derive(Debug, Clone)]
pub struct ColorRect {
    pub color: Color,
    pub offset: Vector2<f32>,
    pub width: u32,
    pub height: u32,
    pub centered: bool,
    pub rotation: f32,
    pub z_index: f32,
}
impl ColorRect {
    pub fn new(color: Color, width: u32, height: u32) -> Self {
        let mut rect = ColorRect::default();
        rect.color = color;
        rect.width = width;
        rect.height = height;

        rect
    }
}
impl Default for ColorRect {
    fn default() -> ColorRect {
        ColorRect {
            color: WHITE,
            offset: Vector2::new(0.0, 0.0),
            width: 32,
            height: 32,
            centered: true,
            rotation: 0.0,
            z_index: 0.0,
        }
    }
}