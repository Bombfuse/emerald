use rapier2d::na::Vector2;

use crate::EmeraldError;

pub struct Polygon {
    points: Vec<Vector2<f32>>,
}

impl Polygon {
    /// Accepts a series of points, that in sequence form segments.
    /// Fails if the given points do not form a valid polygon.
    /// The final point automatically connects to the first point.
    pub fn new(points: Vec<Vector2<f32>>) -> Result<Self, EmeraldError> {
        // TODO: assert points form a complete polygon
        Ok(Self { points })
    }
}
