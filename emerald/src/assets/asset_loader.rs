use crate::asset_key::AssetKey;
use crate::assets::*;
use crate::audio::*;
use crate::ent::load_ent_from_toml;
use crate::font::Font;
use crate::font::FontImage;
use crate::font::FontKey;
use crate::rendering::components::Sprite;
use crate::rendering_engine::RenderingEngine;
use crate::texture::get_texture_key;
use crate::texture::TextureKey;
use crate::*;

use std::collections::HashMap;
use std::ffi::OsStr;

#[cfg(feature = "aseprite")]
use crate::rendering::components::Aseprite;

pub type CustomComponentLoader =
    fn(&mut AssetLoader<'_>, Entity, &mut World, toml::Value, String) -> Result<(), EmeraldError>;

pub type WorldResourceLoader =
    fn(&mut AssetLoader<'_>, &mut World, toml::Value, String) -> Result<(), EmeraldError>;

/// A function defined by the user that handles merge results.
/// Given the base world, the other world, and a mapping of OldEntity -> NewEntity ids.
/// Note: The other world will have had all of its entities removed by this point, but its resources will be in tact.
/// This is so that you can manage resource merging according to your games logic.
pub type WorldMergeHandler =
    fn(&mut World, &mut World, HashMap<Entity, Entity>) -> Result<(), EmeraldError>;

pub struct AssetLoadConfig {
    /// The default configuration to use when loading worlds.
    pub world_load_config: WorldLoadConfig,

    /// A user defined function that handles loading their own custom component definitions.
    pub custom_component_loader: Option<CustomComponentLoader>,

    pub world_resource_loader: Option<WorldResourceLoader>,
}
impl Default for AssetLoadConfig {
    fn default() -> Self {
        Self {
            world_load_config: Default::default(),
            custom_component_loader: None,
            world_resource_loader: None,
        }
    }
}

pub struct AssetLoader<'c> {
    pub(crate) asset_engine: &'c mut AssetEngine,
    rendering_engine: &'c mut RenderingEngine,
    _audio_engine: &'c mut AudioEngine,
}
impl<'c> AssetLoader<'c> {
    pub(crate) fn new(
        asset_engine: &'c mut AssetEngine,
        rendering_engine: &'c mut RenderingEngine,
        _audio_engine: &'c mut AudioEngine,
    ) -> Self {
        AssetLoader {
            asset_engine,
            rendering_engine,
            _audio_engine,
        }
    }

    pub fn set_custom_component_loader(&mut self, custom_component_loader: CustomComponentLoader) {
        self.asset_engine.load_config.custom_component_loader = Some(custom_component_loader);
    }

    pub fn set_world_resource_loader(&mut self, world_resource_loader: WorldResourceLoader) {
        self.asset_engine.load_config.world_resource_loader = Some(world_resource_loader);
    }

    /// Retrieves bytes from the assets directory of the game
    pub fn asset_bytes<T: AsRef<str>>(&mut self, file_path: T) -> Result<Vec<u8>, EmeraldError> {
        let path: &str = file_path.as_ref();

        if let Some(key) = self.asset_engine.get_asset_key_by_label::<Vec<u8>>(path) {
            if let Some(bytes) = self.asset_engine.get_asset::<Vec<u8>>(&key.asset_id) {
                return Ok(bytes.clone());
            }
        }

        let bytes = self.asset_engine.read_asset_file(&path)?;
        Ok(bytes)
    }

    /// Retrieves bytes from a file in the user directory of the game
    pub fn user_bytes<T: AsRef<str>>(&mut self, file_path: T) -> Result<Vec<u8>, EmeraldError> {
        let path: &str = file_path.as_ref();
        if let Some(bytes) = self.asset_engine.get_asset_by_label::<Vec<u8>>(&path) {
            return Ok(bytes.clone());
        }

        let bytes = self.asset_engine.read_user_file(&path)?;
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

        if let Some(key) = self.asset_engine.get_asset_key_by_label::<Font>(&file_path) {
            return Ok(FontKey::new(key, file_path, font_size));
        }

        let font_image = FontImage::gen_image_color(512, 512, Color::new(0, 0, 0, 0));
        let font_texture_key = self.rendering_engine.load_texture_ext(
            file_path,
            &mut self.asset_engine,
            font_image.width as u32,
            font_image.height as u32,
            &font_image.bytes,
        )?;
        let font_bytes = self.asset_bytes(file_path)?;
        let font_settings = fontdue::FontSettings {
            scale: font_size as f32,
            ..Default::default()
        };
        let inner_font = match fontdue::Font::from_bytes(font_bytes, font_settings) {
            Ok(font) => font,
            Err(e) => return Err(EmeraldError::new(e)),
        };
        let font = Font::new(inner_font, font_texture_key.clone(), font_image)?;
        let key = self
            .asset_engine
            .add_asset_with_label(Box::new(font), file_path)?;

        Ok(FontKey::new(key, file_path, font_size))
    }

    pub fn ent<T: AsRef<str>>(
        &mut self,
        world: &mut World,
        path: T,
        transform: Transform,
    ) -> Result<Entity, EmeraldError> {
        let toml = self.string(path)?;
        load_ent_from_toml(self, world, toml, transform)
    }

    pub fn world<T: AsRef<str>>(&mut self, path: T) -> Result<World, EmeraldError> {
        let toml = self.string(path)?;
        load_world(self, toml)
    }

    /// Loads a `.aseprite` file.
    #[cfg(feature = "aseprite")]
    pub fn aseprite<T: AsRef<str>>(&mut self, path: T) -> Result<Aseprite, EmeraldError> {
        let path = path.as_ref();
        let data = self.asset_bytes(path)?;
        Aseprite::new(
            &self.rendering_engine.bind_group_layouts,
            &self.rendering_engine.device,
            &self.rendering_engine.queue,
            self.asset_engine,
            path,
            data,
        )
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

        if let Some(key) = get_texture_key(&mut self.asset_engine, path) {
            return Ok(key);
        }

        let data = self.asset_bytes(path)?;
        self.rendering_engine
            .load_texture(path, &mut self.asset_engine, &data)
    }

    /// Creating render textures is slightly expensive and should be used conservatively.
    /// Please re-use render textures you've created before if possible.
    /// If you need a render texture with a new size, you should create a new render texture.
    pub fn render_texture(&mut self, w: usize, h: usize) -> Result<TextureKey, EmeraldError> {
        self.rendering_engine
            .create_render_texture(w as _, h as _, &mut self.asset_engine)
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

        if let Some(asset_key) = self.asset_engine.get_asset_key_by_label::<Sound>(path) {
            return Ok(SoundKey::new(asset_key, sound_format));
        }

        let sound_bytes = self.asset_bytes(path.clone())?;
        let sound = Sound::new(sound_bytes, sound_format)?;
        let asset_key = self.asset_engine.add_asset(Box::new(sound))?;
        Ok(SoundKey::new(asset_key, sound_format))
    }

    pub fn pack_asset_bytes(
        &mut self,
        name: &str,
        bytes: Vec<u8>,
    ) -> Result<AssetKey, EmeraldError> {
        self.asset_engine
            .add_asset_with_label(Box::new(bytes), name)
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
    use crate::{texture::TextureKey, AssetEngine, AssetLoader};

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum HotReloadAssetType {
        Texture,
    }

    #[derive(Clone)]
    pub struct HotReloadMetadata {
        pub last_modified: std::time::SystemTime,
        pub asset_type: HotReloadAssetType,
    }

    pub(crate) fn on_insert_texture(asset_store: &mut AssetEngine, texture_path: &str) {
        match std::fs::metadata(&texture_path) {
            Ok(metadata) => {
                if let Ok(system_time) = metadata.modified() {
                    let hot_reload_metadata = HotReloadMetadata {
                        last_modified: system_time,
                        asset_type: HotReloadAssetType::Texture,
                    };

                    asset_engine
                        .file_hot_reload_metadata
                        .insert(texture_path.to_string(), hot_reload_metadata);
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
