use math::Vector2;

use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub offset: Vector2,
    pub zoom: f32,
    pub(crate) is_active: bool,
}
impl Default for Camera {
    fn default() -> Camera {
        Camera {
            offset: Vector2::from_int(0, 0),
            zoom: 1.0,
            is_active: false,
        }
    }
}
