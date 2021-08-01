use crate::assets::*;
use crate::audio::*;
use crate::rendering::*;
use crate::*;

use std::ffi::OsStr;


// assets
// user

pub struct AssetLoader<'a> {
    pub(crate) quad_ctx: &'a mut miniquad::Context,
    asset_store: &'a mut AssetStore,
    rendering_engine: &'a mut RenderingEngine,
    _audio_engine: &'a mut AudioEngine,
}
impl<'a> AssetLoader<'a> {
    pub(crate) fn new(
        quad_ctx: &'a mut miniquad::Context,
        asset_store: &'a mut AssetStore,
        rendering_engine: &'a mut RenderingEngine,
        _audio_engine: &'a mut AudioEngine,
    ) -> Self {
        AssetLoader {
            quad_ctx,
            asset_store,
            rendering_engine,
            _audio_engine,
        }
    }

    /// Retrieves bytes from the assets directory of the game
    pub fn asset_bytes<T: Into<String>>(&mut self, file_path: T) -> Result<Vec<u8>, EmeraldError> {
        let path: String = file_path.into();
        if let Some(bytes) = self.asset_store.get_asset_bytes(&path) {
            return Ok(bytes);
        }

        let bytes = self.asset_store.read_asset_file(&path)?;
        self.asset_store.insert_asset_bytes(String::from(path), bytes.clone())?;

        return Ok(bytes);
    }

    /// Retrieves bytes from a file in the user directory of the game
    pub fn user_bytes<T: Into<String>>(&mut self, file_path: T) -> Result<Vec<u8>, EmeraldError> {
        let path: String = file_path.into();
        if let Some(bytes) = self.asset_store.get_user_bytes(&path) {
            return Ok(bytes);
        }

        let bytes = self.asset_store.read_user_file(&path)?;
        self.asset_store.insert_user_bytes(String::from(path), bytes.clone())?;

        return Ok(bytes);
    }

    /// Loads bytes from given path as a string
    pub fn string<T: Into<String>>(&mut self, file_path: T) -> Result<String, EmeraldError> {
        let bytes = self.asset_bytes(file_path)?;
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
        let font_bytes = self.asset_bytes(file_path.clone())?;
        let mut font_settings = fontdue::FontSettings::default();
        font_settings.scale = font_size as f32;
        let inner_font = fontdue::Font::from_bytes(font_bytes, font_settings)?;
        let font = Font::new(key.clone(), font_texture_key.clone(), font_image)?;

        self.asset_store.insert_texture(&mut self.quad_ctx, font_texture_key, font_texture);
        self.asset_store.insert_fontdue_font(key.clone(), inner_font);
        self.asset_store.insert_font(&mut self.quad_ctx, key.clone(), font)?;

        Ok(key)
    }

    pub fn aseprite_with_animations<T: Into<String>>(
        &mut self,
        path_to_texture: T,
        path_to_animations: T,
    ) -> Result<Aseprite, EmeraldError> {
        let texture_path: String = path_to_texture.into();
        let animation_path: String = path_to_animations.into();

        let aseprite_data = self.asset_bytes(animation_path.clone())?;

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

        let data = self.asset_bytes(path.clone())?;
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

    /// Load the sound at the given path into the given mixer.
    /// Returns the sound handle to play the sound with.
    pub fn sound<T: Into<String>>(&mut self, path: T) -> Result<SoundKey, EmeraldError> {
        let path: String = path.into();
        let file_path = std::path::Path::new(&path);
        let sound_format = match file_path.extension().and_then(OsStr::to_str) {
            Some("wav") => SoundFormat::Wav,
            Some("ogg") => SoundFormat::Ogg,
            _ => {
                return Err(EmeraldError::new(format!(
                    "File must be wav or ogg. Found {:?}",
                    file_path
                )))
            }
        };

        let key = SoundKey::new(path.clone(), sound_format);
        if self.asset_store.contains_sound(&key) {
            return Ok(key);
        }
        let sound_bytes = self.asset_bytes(path.clone())?;
        let sound = Sound::new(sound_bytes, sound_format)?;

        if !self.asset_store.contains_sound(&key) {
            self.asset_store.insert_sound(key.clone(), sound);
        }

        Ok(key)
    }

    pub fn pack_asset_bytes(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), EmeraldError> {
        self.asset_store.insert_asset_bytes(name.into(), bytes)?;

        Ok(())
    }

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
