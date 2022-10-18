use crate::{texture::TextureKey, *};

#[derive(Clone, Debug)]
pub struct Sprite {
    pub target: Rectangle,
    pub rotation: f32,
    pub scale: Vector2<f32>,
    pub offset: Vector2<f32>,
    pub visible: bool,
    pub color: Color,
    pub centered: bool,
    pub(crate) texture_key: TextureKey,
    pub z_index: f32,
}
impl Sprite {
    pub fn from_texture(texture_key: TextureKey) -> Self {
        Sprite {
            texture_key,
            ..Default::default()
        }
    }
}
impl Default for Sprite {
    fn default() -> Sprite {
        Sprite {
            target: Rectangle::zeroed(),
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
            offset: Vector2::new(0.0, 0.0),
            color: WHITE,
            centered: true,
            texture_key: TextureKey::default(),
            z_index: 0.0,
            visible: true,
        }
    }
}
