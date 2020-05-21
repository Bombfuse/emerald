use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct Camera2D {
    pub position: Vector2<f32>,
    pub offset: Vector2<f32>,
    pub centered: bool,
    pub view: Vector2<u32>,
}
impl Default for Camera2D {
    fn default() -> Camera2D {
        Camera2D {
            position: Vector2::new(0.0, 0.0),
            offset: Vector2::new(0.0, 0.0),
            centered: true,
            view: Vector2::new(1600, 900),
        }
    }
}