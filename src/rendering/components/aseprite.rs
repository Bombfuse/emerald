use crate::{EmeraldError};
use crate::rendering::*;

use std::collections::HashMap;
use std::fs::File;


#[derive(Clone, Debug)]
pub struct Aseprite {
    pub current_animation: String,
    elapsed_time: f32,
    pub sprite: Sprite,
    pub animations: HashMap<String, AsepriteAnimation>,
}
impl Aseprite {
    pub(crate) fn new(texture_file: File, animation_file: File) -> Result<Aseprite, EmeraldError> {
        let mut aseprite = Aseprite {
            animations: HashMap::new(),
            current_animation: String::from(""),
            elapsed_time: 0.0,
            sprite: Sprite::default(),
        };

        Ok(aseprite)
    }

    pub fn play<T: Into<String>>(&mut self, new_animation: T) {
        self.elapsed_time = 0.0;

        let new_animation: String = new_animation.into();

        if self.current_animation == new_animation {
            return;
        }
        
        self.current_animation = new_animation;
    }
}

#[derive(Clone, Debug)]
pub struct AsepriteAnimation {
    pub from: u16,
    pub to: u16,
    pub direction: AsepriteAnimationDirection,
}

#[derive(Clone, Debug)]
pub enum AsepriteAnimationDirection {
    Forward,
    Reverse
}