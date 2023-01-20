use rapier2d::na::Vector2;

use crate::{Color, EmeraldError, Polygon};

pub struct ColorPolygon {
    color: Color,
    polygon: Polygon,
}
impl ColorPolygon {
    /// Creates a ColorPolygon given
    pub fn new(color: Color, polygon: Polygon) -> Result<Self, EmeraldError> {
        Ok(Self { color, polygon })
    }
}
