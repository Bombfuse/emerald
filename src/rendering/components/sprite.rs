use crate::*;

#[derive(Clone, Debug)]
pub struct Sprite {
    pub target: Rectangle,
    pub position: Vector2<f32>,
    pub rotation: f32,
    pub scale: Vector2<f32>,
    pub offset: Vector2<f32>,
    pub color: Color,
    pub(crate) texture_key: TextureKey,
    pub(crate) z_index: i64,
}
impl Sprite {
    pub fn from_texture(texture_key: TextureKey) -> Self {
        let mut sprite = Sprite::default();
        sprite.texture_key = texture_key;

        sprite
    }
}
impl Default for Sprite {
    fn default() -> Sprite {
        Sprite {
            target: Rectangle::new(0.0, 0.0, 20.0, 20.0),
            position: Vector2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
            offset: Vector2::new(0.0, 0.0),
            color: WHITE,
            texture_key: TextureKey::default(),
            z_index: 0,
        }
    }
}