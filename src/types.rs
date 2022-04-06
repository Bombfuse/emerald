use nanoserde::DeJson;

#[derive(Clone, Copy, Debug, DeJson)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    // Zeroed out rectangle. When a sprite uses a zeroed out rect, it draws the whole sprite.
    pub fn zeroed() -> Self {
        Rectangle::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn is_zero_sized(self) -> bool {
        self.width == 0.0 && self.height == 0.0
    }

    /// Whether or not the given rectangle and this rectangle intersect
    pub fn intersects_with(&self, other: &Rectangle) -> bool {
        self.x <= (other.x + other.width)
            && (self.x + self.width) >= other.x
            && self.y <= other.y + other.height
            && self.y + self.height >= other.y
    }
}
