use crate::assets::*;
use crate::audio::*;
use crate::rendering::*;
use crate::*;

use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use quad_snd::decoder::{read_ogg, read_wav};

pub struct AssetLoader<'a> {
    pub(crate) quad_ctx: &'a mut miniquad::Context,
    asset_store: &'a mut AssetStore,
    rendering_engine: &'a mut RenderingEngine,
}
impl<'a> AssetLoader<'a> {
    pub(crate) fn new(
        quad_ctx: &'a mut miniquad::Context,
        asset_store: &'a mut AssetStore,
        rendering_engine: &'a mut RenderingEngine,
    ) -> Self {
        AssetLoader {
            quad_ctx,
            asset_store,
            rendering_engine,
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

            if let Some(bytes) = self.asset_store.get_bytes(&path) {
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
            if let Some(bytes) = self.asset_store.get_bytes(&path) {
                let bytes = bytes.clone();
                return Ok(bytes);
            }

            let full_path = self.full_path(path.clone())?;
            let file_path: String = full_path.into_os_string().into_string()?;
            let mut file = File::open(file_path)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;

            self.asset_store.insert_bytes(String::from(path), bytes.clone());

            Ok(bytes)
        }
    }

    /// Loads bytes from given path as a string
    pub fn string<T: Into<String>>(&mut self, file_path: T) -> Result<String, EmeraldError> {
        let bytes = self.bytes(file_path)?;
        let string = String::from_utf8(bytes)?;

        Ok(string)
    }

    pub fn font<T: Into<String>>(
        &mut self,
        file_path: T,
        font_size: u32,
    ) -> Result<FontKey, EmeraldError> {
        let file_path: String = file_path.into();
        let key = FontKey::new(file_path.clone(), font_size);

        if let Some(_) = self.asset_store.get_font(&key) {
            return Ok(key);
        }

        let font_image = FontImage::gen_image_color(512, 512, Color::new(0, 0, 0, 0));
        let font_texture_key = TextureKey::new(key.0.clone());
        let font_texture = Texture::from_rgba8(
            &mut self.quad_ctx,
            font_texture_key.clone(),
            font_image.width,
            font_image.height,
            &font_image.bytes,
        )?;
        let font_bytes = self.bytes(file_path.clone())?;
        let mut font_settings = fontdue::FontSettings::default();
        font_settings.scale = font_size as f32;
        let inner_font = fontdue::Font::from_bytes(font_bytes, font_settings)?;
        let font = Font::new(key.clone(), font_texture_key.clone(), font_image)?;

        self.asset_store.insert_texture(&mut self.quad_ctx, font_texture_key, font_texture);
        self.asset_store.insert_fontdue_font(key.clone(), inner_font);
        self.asset_store.insert_font(&mut self.quad_ctx, key.clone(), font)?;

        Ok(key)
    }

    /// TODO(bombfuse): Automatically load texture and animations from a .aseprite
    // fn aseprite() {}

    pub fn aseprite_with_animations<T: Into<String>>(
        &mut self,
        path_to_texture: T,
        path_to_animations: T,
    ) -> Result<Aseprite, EmeraldError> {
        let texture_path: String = path_to_texture.into();
        let animation_path: String = path_to_animations.into();

        let aseprite_data = self.bytes(animation_path.clone())?;

        let sprite = self.sprite(texture_path)?;
        let aseprite = Aseprite::new(sprite, aseprite_data)?;

        Ok(aseprite)
    }

    pub fn texture<T: Into<String>>(&mut self, path: T) -> Result<TextureKey, EmeraldError> {
        let path: String = path.into();
        let key = TextureKey::new(path.clone());

        if let Some(_) = self.asset_store.get_texture(&key) {
            return Ok(key);
        }

        let data = self.bytes(path.clone())?;
        let texture = Texture::new(&mut self.quad_ctx, key.clone(), data)?;
        self.asset_store.insert_texture(&mut self.quad_ctx, key.clone(), texture);

        Ok(key)
    }


    /// Creating render textures is slightly expensive and should be used conservatively.
    /// Please re-use render textures you've created before if possible.
    /// If you need a render texture with a new size, you should create a new render texture.
    pub fn render_texture(&mut self, w: usize, h: usize) -> Result<TextureKey, EmeraldError> {
        self.rendering_engine.create_render_texture(w, h, &mut self.quad_ctx, &mut self.asset_store)
    }

    pub fn sprite<T: Into<String>>(&mut self, path: T) -> Result<Sprite, EmeraldError> {
        let path: String = path.into();
        let texture_key = self.texture(path.clone())?;

        Ok(Sprite::from_texture(texture_key))
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

        let sound_bytes = self.bytes(path)?;
        let sound = match sound_format {
            SoundFormat::Ogg => read_ogg(sound_bytes.as_slice()).unwrap(),
            SoundFormat::Wav => read_wav(sound_bytes.as_slice()).unwrap(),
        };

        Ok(sound)
    }

    pub fn pack_bytes(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), EmeraldError> {
        self.asset_store.insert_bytes(name.into(), bytes);

        Ok(())
    }

    // TODO(bombfuse): This is a quick hack to get the texture into the asset_store. Make this not build a sprite.
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
