use math::Vector2;

use crate::{asset_key::AssetKey, *};

#[derive(Clone, Debug)]
pub struct Sprite {
    // TODO: Make this an Option<Rectangle>, where None means draw the entire texture
    pub target: Rectangle,
    pub rotation: f32,
    pub scale: Vector2,
    pub offset: Vector2,
    pub visible: bool,
    pub color: Color,
    pub centered: bool,
    pub(crate) texture_key: AssetKey,
    pub z_index: f32,
}
impl Sprite {
    pub fn from_texture(texture_key: AssetKey) -> Self {
        Sprite {
            texture_key,
            target: Rectangle::new(0.0, 0.0, 0.0, 0.0),
            rotation: 0.0,
            scale: Vector2::from_float(1.0, 1.0),
            offset: Vector2::from_float(0.0, 0.0),
            color: WHITE,
            centered: true,
            z_index: 0.0,
            visible: true,
        }
    }
}
