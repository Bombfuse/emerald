use std::sync::Arc;

use crate::*;
use crate::{Color, EmeraldError, Rectangle, Vector2, WHITE};

use asefile::AnimationDirection;
use image::DynamicImage;

use super::Sprite;

#[derive(Clone, Debug)]
pub struct Aseprite {
    pub(crate) data: Arc<AsepriteData>,
    pub(crate) current_tag_index: Option<usize>,
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
    pub(crate) fn get_sprite(&self) -> &Sprite {
        &self.get_frame().sprite
    }

    pub(crate) fn new(
        asset_store: &mut AssetEngine,
        path: &str,
        data: Vec<u8>,
    ) -> Result<Self, EmeraldError> {
        let aseprite = asefile::AsepriteFile::read(std::io::Cursor::new(data))?;
        let data = AsepriteData::from_asefile(asset_store, path, aseprite)?;
        Ok(Self::from_data(data))
    }

    pub fn play_animation(
        world: &mut World,
        id: Entity,
        animation: &str,
    ) -> Result<(), EmeraldError> {
        world
            .get::<&mut Aseprite>(id)
            .ok()
            .map(|mut a| a.play(animation));
        Ok(())
    }

    pub(crate) fn from_exported(
        sprite: Sprite,
        animation_json: Vec<u8>,
    ) -> Result<Self, EmeraldError> {
        let animation_json = std::str::from_utf8(&animation_json)?;
        let json_data: json_types::AsepriteData = serde_json::from_str(animation_json)?;
        let data = AsepriteData::from_sprite_and_json(sprite, json_data)?;
        Ok(Self::from_data(data))
    }

    fn from_data(data: AsepriteData) -> Self {
        let data = Arc::new(data);

        Self {
            data,
            elapsed_time: 0.0,
            total_anim_elapsed_time: 0.0,
            frame_counter: 0,
            current_tag_index: None,
            is_looping: false,
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
            offset: Vector2::new(0.0, 0.0),
            color: WHITE,
            centered: true,
            z_index: 0.0,
            visible: true,
        }
    }

    fn get_current_tag(&self) -> Option<&Tag> {
        self.current_tag_index.map(|idx| &self.data.tags[idx])
    }

    fn get_frame(&self) -> &Frame {
        let index = self
            .get_current_tag()
            .map(|tag| tag.get_frame(self.frame_counter))
            .unwrap_or_default();

        &self.data.frames[index]
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
            .tags
            .iter()
            .find(|tag| tag.name == name)
            .map(|tag| tag.duration)
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
        self.data.tags.iter().position(|tag| tag.name == name)
    }

    fn reset(&mut self) {
        self.elapsed_time = 0.0;
        self.total_anim_elapsed_time = 0.0;
        self.frame_counter = 0;
    }

    pub fn add_delta(&mut self, mut delta: f32) {
        self.total_anim_elapsed_time += delta;

        let tag_duration = self
            .get_current_tag()
            .map(|tag| tag.duration)
            .unwrap_or_default();
        if delta >= tag_duration {
            if self.is_looping {
                // Skip entire loops when looping.
                delta %= tag_duration;
            } else {
                // We don't need to process any more than the maximum length.
                delta = tag_duration;
            }
        }
        // Avoid an infinite loop if user passed f32::INFINITY
        if !delta.is_normal() {
            return;
        }
        self.elapsed_time += delta;

        let num_frames_in_tag = self
            .get_current_tag()
            .map(|tag| tag.num_frames())
            .unwrap_or(1);

        loop {
            // If the animation has ended, don't increment the frame counter at
            // all, to avoid overflow.
            if !self.is_looping && self.frame_counter + 1 >= num_frames_in_tag {
                break;
            }

            let frame = self.get_frame();
            if self.elapsed_time < frame.duration {
                break;
            }

            self.elapsed_time -= frame.duration;
            self.frame_counter += 1;

            // Implement looping.
            if self.is_looping && self.frame_counter >= num_frames_in_tag {
                self.frame_counter = 0;
            }
        }
    }
}

pub fn aseprite_update_system(world: &mut World, delta: f32) {
    for (_, aseprite) in world.query::<&mut Aseprite>().iter() {
        aseprite.add_delta(delta);
    }
}

mod json_types {
    use serde::Deserialize;

    #[derive(Copy, Clone, Debug, Deserialize)]
    pub struct AseRect {
        pub(crate) x: u32,
        pub(crate) y: u32,
        pub(crate) w: u32,
        pub(crate) h: u32,
    }

    #[derive(Copy, Clone, Debug, Deserialize)]
    pub struct AseSize {
        pub(crate) w: u32,
        pub(crate) h: u32,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct AsepriteData {
        pub(crate) frames: Vec<AsepriteFrame>,
        pub(crate) meta: AsepriteMeta,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct AsepriteTag {
        pub(crate) name: String,
        pub(crate) from: u32,
        pub(crate) to: u32,
        pub(crate) direction: String,
    }

    #[derive(Copy, Clone, Debug, Deserialize)]
    pub struct AsepriteFrame {
        pub(crate) frame: AseRect,
        pub(crate) duration: u32,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct AsepriteMeta {
        #[serde(rename(deserialize = "frameTags"))]
        pub(crate) frame_tags: Vec<AsepriteTag>,
        pub(crate) size: AseSize,
    }
}

#[derive(Debug)]
struct Tag {
    name: String,
    from: usize,
    to: usize,
    duration: f32,
    direction: AnimationDirection,
}

impl Tag {
    fn new(
        name: String,
        from: usize,
        to: usize,
        frames: &[Frame],
        direction: AnimationDirection,
    ) -> Self {
        let duration = {
            match direction {
                AnimationDirection::Forward | AnimationDirection::Reverse => {
                    frames[from..=to].iter().map(|frame| frame.duration).sum()
                }

                AnimationDirection::PingPong => {
                    if from == to {
                        frames[from].duration
                    } else {
                        let first_frame_duration = frames[from].duration;
                        let last_frame_duration = frames[to].duration;
                        let other_frames_duration = frames[(from + 1)..to]
                            .iter()
                            .map(|frame| frame.duration)
                            .sum::<f32>();

                        first_frame_duration + other_frames_duration * 2.0 + last_frame_duration
                    }
                }
            }
        };

        Self {
            name,
            from,
            to,
            duration,
            direction,
        }
    }

    fn from_asefile(tag: &asefile::Tag, frames: &[Frame]) -> Self {
        Self::new(
            tag.name().to_owned(),
            tag.from_frame() as usize,
            tag.to_frame() as usize,
            frames,
            tag.animation_direction(),
        )
    }

    fn from_json(tag: json_types::AsepriteTag, frames: &[Frame]) -> Result<Self, EmeraldError> {
        let direction = match tag.direction.as_str() {
            "forward" => AnimationDirection::Forward,
            "reverse" => AnimationDirection::Reverse,
            "pingpong" => AnimationDirection::PingPong,
            _ => {
                return Err(EmeraldError::new(format!(
                    "Bad animation direction string {:?}",
                    tag.direction,
                )));
            }
        };

        Ok(Self::new(
            tag.name,
            tag.from as usize,
            tag.to as usize,
            frames,
            direction,
        ))
    }

    fn num_frames(&self) -> usize {
        match self.direction {
            AnimationDirection::Forward | AnimationDirection::Reverse => self.to - self.from + 1,
            AnimationDirection::PingPong => {
                if self.from == self.to {
                    1
                } else {
                    // Each direction of the animation doesn't include one of
                    // the edges, so we use the (length - 1) times two.
                    (self.to - self.from) * 2
                }
            }
        }
    }

    fn get_frame(&self, index: usize) -> usize {
        match self.direction {
            AnimationDirection::Forward => self.from + index,
            AnimationDirection::Reverse => self.to - index,
            AnimationDirection::PingPong => {
                let half_length = self.to - self.from;
                if index < half_length {
                    self.from + index
                } else {
                    self.to - (index - half_length)
                }
            }
        }
    }
}

#[derive(Debug)]
struct Frame {
    sprite: Sprite,
    duration: f32,
}

impl Frame {
    fn from_asefile(
        asset_engine: &mut AssetEngine,
        path: &str,
        frame_index: u32,
        frame: asefile::Frame<'_>,
    ) -> Result<Self, EmeraldError> {
        // let image = DynamicImage::ImageRgba8(frame.image());

        // let mut label = path.to_owned();
        // label.push('#');
        // label.push_str(&frame_index.to_string());

        // let texture_key = if let Some(key) = get_texture_key(asset_engine, &label) {
        //     key
        // } else {
        //     Texture::from_image(
        //         &label,
        //         bind_group_layouts,
        //         asset_engine,
        //         device,
        //         queue,
        //         &image,
        //     )?
        // };

        // Ok(Self {
        //     sprite: Sprite::from_texture(texture_key),
        //     duration: frame.duration() as f32 / 1000.0,
        // })
        todo!("fix aseprite loading")
    }

    fn from_sprite_and_json(
        mut sprite: Sprite,
        sheet_size: &json_types::AseSize,
        frame: json_types::AsepriteFrame,
    ) -> Self {
        let target = &frame.frame;
        let real_y = (sheet_size.h - target.y - target.h) as f32;
        sprite.target = Rectangle::new(target.x as f32, real_y, target.w as f32, target.h as f32);

        let duration = frame.duration as f32 / 1000.0;

        Self { sprite, duration }
    }
}

#[derive(Debug)]
pub(crate) struct AsepriteData {
    frames: Vec<Frame>,
    tags: Vec<Tag>,
}

impl AsepriteData {
    fn from_asefile(
        asset_store: &mut AssetEngine,
        path: &str,
        aseprite: asefile::AsepriteFile,
    ) -> Result<Self, EmeraldError> {
        let frames: Vec<Frame> = (0..aseprite.num_frames())
            .map(|frame_index| {
                let frame = aseprite.frame(frame_index);
                Frame::from_asefile(asset_store, path, frame_index, frame)
            })
            .collect::<Result<_, EmeraldError>>()?;

        let tags = (0..aseprite.num_tags())
            .map(|i| Tag::from_asefile(aseprite.tag(i), &frames))
            .collect();

        Ok(Self { frames, tags })
    }

    fn from_sprite_and_json(
        sprite: Sprite,
        json_data: json_types::AsepriteData,
    ) -> Result<Self, EmeraldError> {
        let sheet_size = &json_data.meta.size;
        let frames: Vec<Frame> = json_data
            .frames
            .into_iter()
            .map(|frame| Frame::from_sprite_and_json(sprite.clone(), sheet_size, frame))
            .collect();

        let tags = json_data
            .meta
            .frame_tags
            .into_iter()
            .map(|tag| Tag::from_json(tag, &frames))
            .collect::<Result<_, _>>()?;

        Ok(Self { frames, tags })
    }
}
