use crate::*;
use crate::rendering::*;
use crate::audio::*;

use std::fs::File;
use std::ffi::OsStr;

pub struct AssetLoader<'a> {
    pub(crate) quad_ctx: &'a mut miniquad::Context,
    rendering_engine: &'a mut RenderingEngine,
    audio_engine: &'a mut AudioEngine,
}
impl<'a> AssetLoader<'a> {
    pub(crate) fn new(quad_ctx: &'a mut miniquad::Context, rendering_engine: &'a mut RenderingEngine, audio_engine: &'a mut AudioEngine) -> Self {
        AssetLoader {
            rendering_engine,
            quad_ctx,
            audio_engine,
        }
    }

    pub fn file<T: Into<String>>(&self, file_path: T) -> Result<File, EmeraldError> {
        let file_path: String = file_path.into();
        let file = File::open(file_path)?;

        Ok(file)
    }

    /// Auto load the sprite sheet from the json
    // pub fn aseprite<T: Into<String>>(&mut self, path_to_json: T) -> Result<Aseprite, EmeraldError> {
    // }

    pub fn aseprite_with_animations<T: Into<String>>(&mut self, path_to_texture: T, path_to_animations: T) -> Result<Aseprite, EmeraldError> {
        let texture_path = path_to_texture.into();
        let animation_path = path_to_animations.into();

        let texture_file = self.file(texture_path.clone())?;
        let animation_file = self.file(animation_path.clone())?;

        self.rendering_engine.aseprite_with_animations(&mut self.quad_ctx,
            texture_file,
            texture_path,
            animation_file,
            animation_path)
    }

    pub fn sprite(&mut self, path: &str) -> Result<Sprite, EmeraldError> {
        let sprite_file = self.file(path)?;
        self.rendering_engine.sprite(&mut self.quad_ctx, sprite_file, path)
    }

    /// Meant to be used for WASM. Packs the textures into the WASM so
    /// that they can be loaded immediately without breaking the API.
    /// 
    /// emd.loader()
    ///     .set_texture(
    ///         "./assets/bunny.png",
    ///         include_bytes!("../static/assets/bunny.png").to_vec()
    ///     );
    /// 
    pub fn pack_texture(&mut self, name: &str, bytes: Vec<u8>) {
        self.rendering_engine.pack_texture(&mut self.quad_ctx, name, bytes)
    }

    pub fn label<T: Into<String>>(&mut self, text: T, font_key: FontKey) -> Result<Label, EmeraldError> {
        let mut label = Label::default();

        label.font = font_key;
        label.text = text.into();

        Ok(Label::default())
    }

    pub fn font(&mut self, path: &str, font_size: u16) -> Result<FontKey, EmeraldError> {
        let file = self.file(path)?;

        self.rendering_engine.font(&mut self.quad_ctx, file, path, font_size)
    }

    pub fn sound<T: Into<String>>(&mut self, path: T) -> Result<Sound, EmeraldError> {
        let path: String = path.into();
        let file_path = std::path::Path::new(&path);

        let sound_format = match file_path.extension().and_then(OsStr::to_str) {
            Some("wav") => SoundFormat::Wav,
            Some("ogg") => SoundFormat::Ogg,
            _ => return Err(EmeraldError::new(format!("Unable to parse sound from {:?}", file_path))),
        };

        let sound_file = self.file(path)?;

        self.audio_engine.load(sound_file, sound_format)
    }
}