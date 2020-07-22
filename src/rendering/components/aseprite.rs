use crate::{EmeraldError, Rectangle};
use crate::rendering::*;

use nanoserde::{DeJson, SerJson};
use std::collections::HashMap;
use std::fs::File;


#[derive(Clone, Debug)]
pub struct Aseprite {
    pub current_animation: String,
    elapsed_time: f32,
    pub sprite: Sprite,
    pub animations: HashMap<String, Animation>,
}
impl Aseprite {
    pub(crate) fn new(sprite: Sprite, _animation_file: File) -> Result<Aseprite, EmeraldError> {
        let aseprite = Aseprite {
            animations: HashMap::new(),
            current_animation: String::from(""),
            elapsed_time: 0.0,
            sprite,
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

pub struct AsepriteAnimation {

}

#[derive(Clone, Debug, DeJson)]
struct Frame { 
    file_name: String,
    frame: Rectangle,
    duration: u32,
    #[nserde(rename = "spriteSourceSize")]
    sprite_source_size: Rectangle,
    #[nserde(rename = "sourceSize")]
    source_size: Rectangle,
    rotated: bool,
    trimmed: bool,
}

#[derive(Clone, Debug, DeJson)]
pub struct Animation {
    pub from: u16,
    pub to: u16,
    pub direction: AnimationDirection,
}

#[derive(Clone, Debug, DeJson)]
pub enum AnimationDirection {
    Forward,
    Reverse
}