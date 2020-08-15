use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub offset: Vector2<f32>,
    pub centered: bool,
    pub zoom: f32,
}
impl Default for Camera {
    fn default() -> Camera {
        Camera {
            offset: Vector2::new(0.0, 0.0),
            centered: true,
            zoom: 1.0,
        }
    }
}