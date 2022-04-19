use std::sync::Arc;

use crate::*;
use crate::{Color, EmeraldError, Rectangle, Vector2, WHITE};

use nanoserde::DeJson;

use types::*;

#[derive(Clone, Debug)]
pub struct Aseprite {
    pub(crate) data: Arc<AsepriteData>,
    pub(crate) current_tag_index: Option<usize>,
    pub(crate) sprite: Sprite,
    pub(crate) elapsed_time: f32,
    pub(crate) total_anim_elapsed_time: f32,
    pub(crate) is_looping: bool,
    frame_counter: usize,

    pub rotation: f32,
    pub scale: Vector2<f32>,
    pub offset: Vector2<f32>,
    pub visible: bool,
    pub color: Color,
    pub centered: bool,
    pub z_index: f32,
}
impl Aseprite {
    /// Update the inner sprite to reflect the state of the Aseprite.
    /// This should be done before each time the Aseprite is drawn.
    pub(crate) fn update(&mut self) {
        let sheet_size = &self.data.meta.size;
        let frame = self.get_frame();
        let target = &frame.frame;
        let real_y = (sheet_size.h - target.y - target.h) as f32;

        self.sprite.target =
            Rectangle::new(target.x as f32, real_y, target.w as f32, target.h as f32);
    }

    pub(crate) fn new(sprite: Sprite, animation_json: Vec<u8>) -> Result<Aseprite, EmeraldError> {
        let animation_json = std::str::from_utf8(&animation_json)?;
        let data: AsepriteData = DeJson::deserialize_json(animation_json)?;
        let data = Arc::new(data);

        let aseprite = Aseprite {
            data,
            elapsed_time: 0.0,
            total_anim_elapsed_time: 0.0,
            frame_counter: 0,
            current_tag_index: None,
            sprite,
            is_looping: false,
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
            offset: Vector2::new(0.0, 0.0),
            color: WHITE,
            centered: true,
            z_index: 0.0,
            visible: true,
        };

        Ok(aseprite)
    }

    fn get_current_tag(&self) -> Option<&AsepriteTag> {
        self.current_tag_index
            .map(|idx| &self.data.meta.frame_tags[idx])
    }

    pub fn get_animation_name(&self) -> &str {
        self.get_current_tag()
            .map(|tag| tag.name.as_str())
            .unwrap_or("")
    }

    pub fn get_elapsed_time(&self) -> f32 {
        self.total_anim_elapsed_time
    }

    /// Returns the length of the animation given in seconds.
    /// Returns 0.0 if the animation doesn't exist.
    pub fn get_anim_length<T: AsRef<str>>(&self, name: T) -> f32 {
        let name: &str = name.as_ref();

        self.data
            .meta
            .frame_tags
            .iter()
            .find(|tag| tag.name == name)
            .map(|tag| {
                let total_time: u32 = (tag.from..=tag.to)
                    .filter_map(|i| self.data.frames.get(i as usize))
                    .map(|frame| frame.duration)
                    .sum();

                total_time as f32 / 1000.0
            })
            .unwrap_or(0.0)
    }

    pub fn play<T: AsRef<str>>(&mut self, new_animation: T) -> Result<(), EmeraldError> {
        self.is_looping = false;
        self.reset();
        self.set_tag(new_animation)
    }

    pub fn play_and_loop<T: AsRef<str>>(&mut self, new_animation: T) -> Result<(), EmeraldError> {
        self.is_looping = true;
        self.reset();
        self.set_tag(new_animation)
    }

    fn set_tag<T: AsRef<str>>(&mut self, animation_name: T) -> Result<(), EmeraldError> {
        let animation_name: &str = animation_name.as_ref();

        if let Some(idx) = self.find_tag(animation_name) {
            self.current_tag_index = Some(idx);
        } else {
            return Err(EmeraldError::new(format!(
                "Animation {} does not exist.",
                animation_name
            )));
        }

        Ok(())
    }

    fn find_tag(&self, name: &str) -> Option<usize> {
        self.data
            .meta
            .frame_tags
            .iter()
            .position(|tag| tag.name == name)
    }

    fn reset(&mut self) {
        self.elapsed_time = 0.0;
        self.total_anim_elapsed_time = 0.0;
        self.frame_counter = 0;
    }

    pub fn add_delta(&mut self, delta: f32) {
        self.elapsed_time += delta;
        self.total_anim_elapsed_time += delta;

        let num_frames_in_tag: u32 = self
            .get_current_tag()
            .map(|tag| tag.to - tag.from)
            .unwrap_or_default();

        loop {
            let frame = self.get_frame();
            let duration = frame.duration as f32 / 1000.0;
            if self.elapsed_time < duration {
                break;
            }

            self.elapsed_time -= duration;
            self.frame_counter += 1;

            if self.frame_counter as u32 > num_frames_in_tag {
                if self.is_looping {
                    self.frame_counter = 0;
                } else {
                    self.frame_counter = num_frames_in_tag as usize;
                }
            }
        }
    }

    fn get_frame(&self) -> &AsepriteFrame {
        let cur_tag_start: u32 = self
            .get_current_tag()
            .map(|tag| tag.from)
            .unwrap_or_default();

        &self.data.frames[cur_tag_start as usize + self.frame_counter]
    }
}

pub fn aseprite_update_system(world: &mut World, delta: f32) {
    for (_, aseprite) in world.query::<&mut Aseprite>().iter() {
        aseprite.add_delta(delta);
    }
}

pub(crate) mod types {
    use nanoserde::DeJson;

    #[derive(Copy, Clone, Debug, DeJson)]
    pub struct AseRect {
        pub(crate) x: u32,
        pub(crate) y: u32,
        pub(crate) w: u32,
        pub(crate) h: u32,
    }

    #[derive(Copy, Clone, Debug, DeJson)]
    pub struct AseSize {
        #[nserde(rename = "w")]
        pub(crate) _w: u32,
        pub(crate) h: u32,
    }

    #[derive(Clone, Debug, DeJson)]
    pub struct AsepriteData {
        pub(crate) frames: Vec<AsepriteFrame>,
        pub(crate) meta: AsepriteMeta,
    }

    #[derive(Clone, Debug, DeJson)]
    pub struct AsepriteTag {
        pub(crate) name: String,
        pub(crate) from: u32,
        pub(crate) to: u32,
        #[nserde(rename = "direction")]
        pub(crate) _direction: String,
    }
    impl Default for AsepriteTag {
        fn default() -> AsepriteTag {
            AsepriteTag {
                name: String::from(""),
                from: 0,
                to: 0,
                _direction: String::from("forward"),
            }
        }
    }

    #[derive(Copy, Clone, Debug, DeJson)]
    pub struct AsepriteFrame {
        pub(crate) frame: AseRect,
        #[nserde(rename = "rotated")]
        _rotated: bool,
        #[nserde(rename = "trimmed")]
        _trimmed: bool,
        pub(crate) duration: u32,
        #[nserde(rename = "spriteSourceSize")]
        _sprite_source_size: AseRect,
        #[nserde(rename = "sourceSize")]
        _source_size: AseSize,
    }

    #[derive(Clone, Debug, DeJson)]
    pub struct AsepriteMeta {
        #[nserde(rename = "format")]
        _format: String,
        pub(crate) size: AseSize,
        #[nserde(rename = "scale")]
        _scale: String,
        #[nserde(rename = "frameTags")]
        pub(crate) frame_tags: Vec<AsepriteTag>,
    }
}
