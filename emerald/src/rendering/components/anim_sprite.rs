use std::collections::HashMap;

use crate::{asset_key::AssetKey, Rectangle, Translation};

pub struct AnimSprite {
    textures: HashMap<u8, AssetKey>,
    animations: HashMap<String, Animation>,
    data: AnimSpriteData,
}
impl AnimSprite {
    pub fn play_animation(&mut self) {}
    pub fn play_and_loop(&mut self) {}
    pub fn play(&mut self) {}
    pub fn play_ext(&mut self) {}
}

struct AnimSpriteData {
    current_animation: Option<String>,
    elapsed_time: f32,
}

struct Animation {
    frames: Vec<AnimationFrame>,
}

struct AnimationFrame {
    /// Which texture to use on the AnimSprite
    texture_id: u8,

    /// Target rect on the given texture
    target: Rectangle,

    /// Offset to draw this frame
    offset: Translation,

    /// How long
    duration_ms: f32,
}
