use std::convert::TryInto;
use std::sync::Arc;

use crate::*;
use crate::{Color, EmeraldError, Rectangle, Vector2, WHITE};

use miniquad::Context;
use nanoserde::DeJson;

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
        ctx: &mut Context,
        asset_store: &mut AssetStore,
        path: &str,
        data: Vec<u8>,
    ) -> Result<Self, EmeraldError> {
        let aseprite = asefile::AsepriteFile::read(std::io::Cursor::new(data))?;
        let data = AsepriteData::from_asefile(ctx, asset_store, path, aseprite)?;
        Ok(Self::from_data(data))
    }

    pub(crate) fn from_exported(
        sprite: Sprite,
        animation_json: Vec<u8>,
    ) -> Result<Self, EmeraldError> {
        let animation_json = std::str::from_utf8(&animation_json)?;
        let json_data: json_types::AsepriteData = DeJson::deserialize_json(animation_json)?;
        let data = AsepriteData::from_sprite_and_json(sprite, json_data);
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
        let cur_tag_start = self
            .get_current_tag()
            .map(|tag| tag.from)
            .unwrap_or_default();

        &self.data.frames[cur_tag_start + self.frame_counter]
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
            .map(|tag| {
                (tag.from..=tag.to)
                    .filter_map(|i| self.data.frames.get(i))
                    .map(|frame| frame.duration)
                    .sum()
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
        self.data.tags.iter().position(|tag| tag.name == name)
    }

    fn reset(&mut self) {
        self.elapsed_time = 0.0;
        self.total_anim_elapsed_time = 0.0;
        self.frame_counter = 0;
    }

    pub fn add_delta(&mut self, delta: f32) {
        self.elapsed_time += delta;
        self.total_anim_elapsed_time += delta;

        let num_frames_in_tag = self
            .get_current_tag()
            .map(|tag| tag.to - tag.from)
            .unwrap_or_default();

        loop {
            let frame = self.get_frame();
            if self.elapsed_time < frame.duration {
                break;
            }

            self.elapsed_time -= frame.duration;
            self.frame_counter += 1;

            if self.frame_counter > num_frames_in_tag {
                if self.is_looping {
                    self.frame_counter = 0;
                } else {
                    self.frame_counter = num_frames_in_tag;
                }
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

#[derive(Debug)]
struct Tag {
    name: String,
    from: usize,
    to: usize,
}

impl Tag {
    fn from_asefile(tag: &asefile::Tag) -> Self {
        Self {
            name: tag.name().to_owned(),
            from: tag.from_frame() as usize,
            to: tag.to_frame() as usize,
        }
    }
}

impl From<json_types::AsepriteTag> for Tag {
    fn from(tag: json_types::AsepriteTag) -> Self {
        Self {
            name: tag.name,
            from: tag.from as usize,
            to: tag.to as usize,
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
        ctx: &mut Context,
        asset_store: &mut AssetStore,
        path: &str,
        frame_index: u32,
        frame: asefile::Frame<'_>,
    ) -> Result<Self, EmeraldError> {
        let image = frame.image();
        let image = image::imageops::flip_vertical(&image);
        let texture_key = {
            let mut key = path.to_owned();
            key.push('#');
            key.push_str(&frame_index.to_string());
            TextureKey::new(key)
        };
        let texture = Texture::from_rgba8(
            ctx,
            texture_key.clone(),
            image.width().try_into().unwrap(),
            image.height().try_into().unwrap(),
            &image,
        )?;
        asset_store.insert_texture(texture_key.clone(), texture);

        Ok(Self {
            sprite: Sprite::from_texture(texture_key),
            duration: frame.duration() as f32 / 1000.0,
        })
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
        ctx: &mut Context,
        asset_store: &mut AssetStore,
        path: &str,
        aseprite: asefile::AsepriteFile,
    ) -> Result<Self, EmeraldError> {
        let frames = (0..aseprite.num_frames())
            .map(|frame_index| {
                let frame = aseprite.frame(frame_index);
                Frame::from_asefile(ctx, asset_store, path, frame_index, frame)
            })
            .collect::<Result<_, EmeraldError>>()?;

        let tags = (0..aseprite.num_tags())
            .map(|i| Tag::from_asefile(aseprite.tag(i)))
            .collect();

        Ok(Self { frames, tags })
    }

    fn from_sprite_and_json(sprite: Sprite, json_data: json_types::AsepriteData) -> Self {
        let sheet_size = &json_data.meta.size;
        let frames = json_data
            .frames
            .into_iter()
            .map(|frame| Frame::from_sprite_and_json(sprite.clone(), sheet_size, frame))
            .collect();

        let tags = json_data
            .meta
            .frame_tags
            .into_iter()
            .map(Tag::from)
            .collect();

        Self { frames, tags }
    }
}
