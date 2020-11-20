use crate::assets::*;
use crate::audio::*;
use crate::rendering::*;
use crate::*;

use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub struct AssetLoader<'a> {
    pub(crate) quad_ctx: &'a mut miniquad::Context,
    rendering_engine: &'a mut RenderingEngine,
    audio_engine: &'a mut AudioEngine,
    cache: &'a mut Cache,
}
impl<'a> AssetLoader<'a> {
    pub(crate) fn new(
        quad_ctx: &'a mut miniquad::Context,
        rendering_engine: &'a mut RenderingEngine,
        audio_engine: &'a mut AudioEngine,
        cache: &'a mut Cache,
    ) -> Self {
        AssetLoader {
            rendering_engine,
            quad_ctx,
            audio_engine,
            cache,
        }
    }

    fn full_path<T: Into<String>>(&self, file_path: T) -> Result<PathBuf, EmeraldError> {
        let current_dir = std::env::current_dir()?;

        Ok(current_dir.join(file_path.into()))
    }

    pub fn bytes<T: Into<String>>(&mut self, file_path: T) -> Result<Vec<u8>, EmeraldError> {
        #[cfg(target_arch = "wasm32")]
        {
            let path: String = file_path.into();

            if let Some(bytes) = self.cache.data.get(&path) {
                return Ok(bytes.clone());
            }

            Err(EmeraldError::new(format!(
                "Unable to get bytes for {}",
                path
            )))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let path: String = file_path.into();
            if let Some(bytes) = self.cache.data.get(&path) {
                let bytes = bytes.clone();
                return Ok(bytes);
            }

            let full_path = self.full_path(path.clone())?;
            let file_path: String = full_path.into_os_string().into_string()?;
            let mut file = File::open(file_path)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;

            self.cache.data.insert(String::from(path), bytes.clone());

            Ok(bytes)
        }
    }

    /// Loads bytes from given path as a string
    pub fn string<T: Into<String>>(&mut self, file_path: T) -> Result<String, EmeraldError> {
        let bytes = self.bytes(file_path)?;
        let string = String::from_utf8(bytes)?;

        Ok(string)
    }

    pub fn font<T: Into<String>>(&mut self, file_path: T, font_size: u32) -> Result<FontKey, EmeraldError> {
        let file_path: String = file_path.into();

        let key = FontKey::new(file_path.clone(), font_size);

        if self.rendering_engine.fonts.contains_key(&key) {
            return Ok(key);
        }

        let font_data = self.bytes(file_path.clone())?;

        self.rendering_engine
            .font(&mut self.quad_ctx, file_path, font_data, font_size)
    }

    /// TODO(bombfuse): Automatically load the spritesheet from the aseprite json file
    // fn aseprite() {}

    pub fn aseprite_with_animations<T: Into<String>>(
        &mut self,
        path_to_texture: T,
        path_to_animations: T,
    ) -> Result<Aseprite, EmeraldError> {
        let texture_path: String = path_to_texture.into();
        let animation_path: String = path_to_animations.into();

        let texture_data = self.bytes(texture_path.clone())?;
        let aseprite_data = self.bytes(animation_path.clone())?;

        self.rendering_engine.aseprite_with_animations(
            &mut self.quad_ctx,
            texture_data,
            texture_path,
            aseprite_data,
            animation_path,
        )
    }

    pub fn sprite<T: Into<String>>(&mut self, path: T) -> Result<Sprite, EmeraldError> {
        let path: String = path.into();

        match self.rendering_engine.sprite(path.clone()) {
            Ok(sprite) => Ok(sprite),
            Err(_e) => {
                let sprite_data = self.bytes(path.clone())?;
                self.rendering_engine
                    .sprite_from_data(&mut self.quad_ctx, sprite_data, path)
            }
        }
    }

    pub fn sound<T: Into<String>>(&mut self, path: T) -> Result<Sound, EmeraldError> {
        let path: String = path.into();
        let file_path = std::path::Path::new(&path);

        let sound_format = match file_path.extension().and_then(OsStr::to_str) {
            Some("wav") => SoundFormat::Wav,
            Some("ogg") => SoundFormat::Ogg,
            _ => {
                return Err(EmeraldError::new(format!(
                    "Unable to parse sound from {:?}",
                    file_path
                )))
            }
        };

        let sound_data = self.bytes(path)?;

        self.audio_engine.load(sound_data, sound_format)
    }

    pub fn world<T: Into<String>>(&mut self, path: T) -> Result<EmeraldWorld, EmeraldError> {
        let bytes = self.bytes(path)?;
        let json = std::str::from_utf8(&bytes).unwrap();

        crate::assets::loading::deserialize_world_from_json(&String::from(json), self)
    }

    pub fn pack_bytes(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), EmeraldError> {
        self.cache.data.insert(name.into(), bytes);

        Ok(())
    }

    // TODO(bombfuse): This is a quick hack to get the texture into the cache. Make this not build a sprite.
    pub fn preload_texture<T: Into<String>>(&mut self, name: T) -> Result<(), EmeraldError> {
        let name: String = name.into();

        if let Ok(_sprite) = self.sprite(name.clone()) {
            return Ok(());
        }

        Err(EmeraldError::new(format!(
            "Unable to preload texture {}",
            name
        )))
    }
}
