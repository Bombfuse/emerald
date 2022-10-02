use crate::assets::*;
use crate::audio::*;
use crate::ent::load_ent;
use crate::rendering::*;
use crate::*;

use std::ffi::OsStr;

pub struct AssetLoader<'c> {
    pub(crate) quad_ctx: &'c mut miniquad::Context,
    pub(crate) asset_store: &'c mut AssetStore,
    rendering_engine: &'c mut RenderingEngine,
    _audio_engine: &'c mut AudioEngine,
}
impl<'c> AssetLoader<'c> {
    pub(crate) fn new(
        quad_ctx: &'c mut miniquad::Context,
        asset_store: &'c mut AssetStore,
        rendering_engine: &'c mut RenderingEngine,
        _audio_engine: &'c mut AudioEngine,
    ) -> Self {
        AssetLoader {
            quad_ctx,
            asset_store,
            rendering_engine,
            _audio_engine,
        }
    }

    /// Retrieves bytes from the assets directory of the game
    pub fn asset_bytes<T: AsRef<str>>(&mut self, file_path: T) -> Result<Vec<u8>, EmeraldError> {
        let path: &str = file_path.as_ref();
        if let Some(bytes) = self.asset_store.get_asset_bytes(&path) {
            return Ok(bytes);
        }

        let bytes = self.asset_store.read_asset_file(&path)?;

        Ok(bytes)
    }

    /// Retrieves bytes from a file in the user directory of the game
    pub fn user_bytes<T: AsRef<str>>(&mut self, file_path: T) -> Result<Vec<u8>, EmeraldError> {
        let path: &str = file_path.as_ref();
        if let Some(bytes) = self.asset_store.get_user_bytes(&path) {
            return Ok(bytes);
        }

        let bytes = self.asset_store.read_user_file(&path)?;
        Ok(bytes)
    }

    /// Loads bytes from given path as a string
    pub fn string<T: AsRef<str>>(&mut self, file_path: T) -> Result<String, EmeraldError> {
        let bytes = self.asset_bytes(file_path)?;
        let string = String::from_utf8(bytes)?;

        Ok(string)
    }

    pub fn font<T: AsRef<str>>(
        &mut self,
        file_path: T,
        font_size: u32,
    ) -> Result<FontKey, EmeraldError> {
        let file_path: &str = file_path.as_ref();
        let key = FontKey::new(file_path.clone(), font_size);

        if self.asset_store.get_font(&key).is_some() {
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
        let font_bytes = self.asset_bytes(file_path)?;

        let font_settings = fontdue::FontSettings {
            scale: font_size as f32,
            ..Default::default()
        };
        let inner_font = fontdue::Font::from_bytes(font_bytes, font_settings)?;
        let font = Font::new(key.clone(), font_texture_key.clone(), font_image)?;

        self.asset_store
            .insert_texture(font_texture_key, font_texture);
        self.asset_store
            .insert_fontdue_font(key.clone(), inner_font);
        self.asset_store
            .insert_font(&mut self.quad_ctx, key.clone(), font)?;

        Ok(key)
    }

    pub fn ent<T: AsRef<str>>(
        &mut self,
        world: &mut World,
        transform: Transform,
        path: T,
    ) -> Result<Entity, EmeraldError> {
        let toml = self.string(path)?;
        load_ent(self, world, transform, toml)
    }

    /// Loads a `.aseprite` file.
    #[cfg(feature = "aseprite")]
    pub fn aseprite<T: AsRef<str>>(&mut self, path: T) -> Result<Aseprite, EmeraldError> {
        let path = path.as_ref();
        let data = self.asset_bytes(path)?;
        Aseprite::new(self.quad_ctx, self.asset_store, path, data)
    }

    /// Loads an exported Aseprite sprite sheet. The animations json file should
    /// have been exported in the "Array" mode.
    #[cfg(feature = "aseprite")]
    pub fn aseprite_with_animations<T: AsRef<str>>(
        &mut self,
        path_to_texture: T,
        path_to_animations: T,
    ) -> Result<Aseprite, EmeraldError> {
        let texture_path: &str = path_to_texture.as_ref();
        let animation_path: &str = path_to_animations.as_ref();

        let aseprite_data = self.asset_bytes(animation_path)?;

        let sprite = self.sprite(texture_path)?;
        let aseprite = Aseprite::from_exported(sprite, aseprite_data)?;

        Ok(aseprite)
    }

    pub fn texture<T: AsRef<str>>(&mut self, path: T) -> Result<TextureKey, EmeraldError> {
        let path: &str = path.as_ref();
        let key = TextureKey::new(path.clone());

        if self.asset_store.get_texture(&key).is_some() {
            return Ok(key);
        }

        let data = self.asset_bytes(path)?;
        let texture = Texture::new(&mut self.quad_ctx, key.clone(), data)?;
        self.asset_store.insert_texture(key.clone(), texture);

        Ok(key)
    }

    /// Creating render textures is slightly expensive and should be used conservatively.
    /// Please re-use render textures you've created before if possible.
    /// If you need a render texture with a new size, you should create a new render texture.
    pub fn render_texture(&mut self, w: usize, h: usize) -> Result<TextureKey, EmeraldError> {
        self.rendering_engine
            .create_render_texture(w, h, &mut self.quad_ctx, &mut self.asset_store)
    }

    pub fn sprite<T: AsRef<str>>(&mut self, path: T) -> Result<Sprite, EmeraldError> {
        let path: &str = path.as_ref();
        let texture_key = self.texture(path)?;

        Ok(Sprite::from_texture(texture_key))
    }

    /// Load the sound at the given path into the given mixer.
    /// Returns the sound handle to play the sound with.
    pub fn sound<T: AsRef<str>>(&mut self, path: T) -> Result<SoundKey, EmeraldError> {
        let path: &str = path.as_ref();
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

    pub fn preload_texture<T: AsRef<str>>(&mut self, name: T) -> Result<(), EmeraldError> {
        let name: &str = name.as_ref();

        if let Ok(_sprite) = self.sprite(name.clone()) {
            return Ok(());
        }

        Err(EmeraldError::new(format!(
            "Unable to preload texture {}",
            name
        )))
    }

    pub fn hotreload(&mut self) {
        #[cfg(feature = "hotreload")]
        hotreload::run(self)
    }
}

#[cfg(feature = "hotreload")]
pub(crate) mod hotreload {
    use crate::{AssetLoader, AssetStore, TextureKey};

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum HotReloadAssetType {
        Texture,
    }

    #[derive(Clone)]
    pub struct HotReloadMetadata {
        pub last_modified: std::time::SystemTime,
        pub asset_type: HotReloadAssetType,
    }

    pub(crate) fn on_insert_texture(asset_store: &mut AssetStore, texture_path: &str) {
        match std::fs::metadata(&texture_path) {
            Ok(metadata) => {
                if let Ok(system_time) = metadata.modified() {
                    let hot_reload_metadata = HotReloadMetadata {
                        last_modified: system_time,
                        asset_type: HotReloadAssetType::Texture,
                    };

                    asset_store
                        .file_hot_reload_metadata
                        .insert(texture_path, hot_reload_metadata);
                }
            }
            Err(_) => {}
        }
    }

    pub(crate) fn run(loader: &mut AssetLoader<'_>) {
        let mut updates = Vec::new();

        for (path, hot_reload_metadata) in &loader.asset_store.file_hot_reload_metadata {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(new_system_time) = metadata.modified() {
                    if let Ok(duration) =
                        new_system_time.duration_since(hot_reload_metadata.last_modified)
                    {
                        if duration.as_millis() > 0 {
                            updates.push((
                                path.clone(),
                                hot_reload_metadata.asset_type,
                                new_system_time,
                            ));
                        }
                    }
                }
            }
        }

        for (mut path, asset_type, new_system_time) in updates {
            match asset_type {
                HotReloadAssetType::Texture => {
                    let asset_root_folder_path = loader.asset_store.get_asset_folder_root();
                    let relative_path = path.split_off(asset_root_folder_path.len());
                    if loader
                        .asset_store
                        .remove_texture(TextureKey::new(relative_path.clone()), false)
                        .is_some()
                        && loader.texture(relative_path).is_ok()
                    {
                        if let Some(mut hotreload_metadata) =
                            loader.asset_store.file_hot_reload_metadata.get_mut(&path)
                        {
                            hotreload_metadata.last_modified = new_system_time;
                        }
                    }
                }
            }
        }
    }
}
