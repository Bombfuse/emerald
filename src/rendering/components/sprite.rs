use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub target: Rectangle,
    pub position: Vector2<f32>,
    pub rotation: f32,
    pub scale: Vector2<f32>,
    pub offset: Vector2<f32>,
    pub color: Color,
    pub(crate) texture_key: TextureKey,
}
impl Sprite {
}
impl Default for Sprite {
    fn default() -> Sprite {
        Sprite {
            target: Rectangle::zeroed(),
            position: Vector2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
            offset: Vector2::new(0.0, 0.0),
            color: WHITE,
            texture_key: TextureKey::default(),
        }
    }
}