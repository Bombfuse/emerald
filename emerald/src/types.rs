use serde::{Deserialize, Serialize};

pub use hecs::Entity;
pub use rapier2d::na::Vector2;
pub use rapier2d::na::Vector3;

pub mod polygon;
pub use polygon::*;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a `Rectangle` from its bottom-left point and its size.
    pub fn from_point_and_size(
        point: impl Into<crate::Vector2<f32>>,
        size: impl Into<crate::Vector2<f32>>,
    ) -> Self {
        let point = point.into();
        let size = size.into();

        Self {
            x: point.x,
            y: point.y,
            width: size.x,
            height: size.y,
        }
    }

    // Zeroed out rectangle. When a sprite uses a zeroed out rect, it draws the whole sprite.
    pub fn zeroed() -> Self {
        Rectangle::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn is_zero_sized(self) -> bool {
        self.width == 0.0 && self.height == 0.0
    }

    /// An alias for self.x
    #[inline]
    pub fn left(&self) -> f32 {
        self.x
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// An alias for self.y
    #[inline]
    pub fn bottom(&self) -> f32 {
        self.y
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.y + self.height
    }

    #[inline]
    pub fn bottom_left(&self) -> Vector2<f32> {
        Vector2::new(self.left(), self.bottom())
    }

    #[inline]
    pub fn size(&self) -> crate::Vector2<f32> {
        Vector2::new(self.width, self.height)
    }

    #[inline]
    pub fn center(&self) -> crate::Vector2<f32> {
        self.bottom_left() + self.size() / 2.0
    }

    /// Whether or not the given rectangle and this rectangle intersect
    pub fn intersects_with(&self, other: &Rectangle) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

#[cfg(test)]
pub mod tests {
    use crate::Rectangle;

    #[test]
    fn exact_overlap_intersects() {
        let rect_a = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        let rect_b = rect_a.clone();

        assert!(rect_a.intersects_with(&rect_b));
        assert!(rect_b.intersects_with(&rect_a));
    }

    #[test]
    fn rect_intersects_halfway() {
        let rect_a = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        let mut rect_b = rect_a.clone();
        rect_b.x = 5.0;
        rect_b.y = 5.0;

        assert!(rect_a.intersects_with(&rect_b));
        assert!(rect_b.intersects_with(&rect_a));
    }

    #[test]
    fn rect_intersects_halfway_negative() {
        let rect_a = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        let mut rect_b = rect_a.clone();
        rect_b.x = -5.0;
        rect_b.y = -5.0;

        assert!(rect_a.intersects_with(&rect_b));
        assert!(rect_b.intersects_with(&rect_a));
    }
}
