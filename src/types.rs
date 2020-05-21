#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}
impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rectangle { x, y, width, height }
    }

    // Zeroed out rectangle. When a sprite uses a zeroed out rect, it draws the whole sprite.
    pub fn zeroed() -> Self {
        Rectangle::new(0.0, 0.0, 0.0, 0.0)
    }
}