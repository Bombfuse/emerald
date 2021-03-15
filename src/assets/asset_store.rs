use crate::{EmeraldError};
use crate::rendering::*;

use miniquad::Context;
use std::collections::HashMap;

const INITIAL_TEXTURE_STORAGE_CAPACITY: usize = 100;
const INITIAL_FONT_STORAGE_CAPACITY: usize = 100;
// const INITIAL_SOUND_STORAGE_CAPACITY: usize = 100;

/// The AssetStore stores all Textures, Fonts, and Audio for the game.
/// It stores the data contiguously, and does caching internally.
/// Assets can be loaded via the `AssetLoader` and inserted into the AssetStore.
/// Assets can be manually removed from the store if memory management becomes a concern.
pub(crate) struct AssetStore {
    bytes: HashMap<String, Vec<u8>>,

    fonts: Vec<Font>,
    fontdue_fonts: Vec<fontdue::Font>,
    textures: Vec<Texture>,

    fontdue_key_map: HashMap<FontKey, usize>,
    font_key_map: HashMap<FontKey, usize>,
    texture_key_map: HashMap<TextureKey, usize>,

    _should_reset_fontdue_font_key_map: bool,
    _should_reset_font_key_map: bool,
    _should_reset_texture_key_map: bool,
}
impl AssetStore {
    pub fn new(ctx: &mut Context) -> Self {
        let mut texture_key_map = HashMap::new();
        let default_texture = Texture::default(ctx).unwrap();
        texture_key_map.insert(TextureKey::default(), 0);

        let mut textures = Vec::with_capacity(INITIAL_TEXTURE_STORAGE_CAPACITY);
        textures.push(default_texture);

        AssetStore {
            bytes: HashMap::new(),
            fontdue_fonts: Vec::with_capacity(INITIAL_FONT_STORAGE_CAPACITY),
            fonts: Vec::with_capacity(INITIAL_FONT_STORAGE_CAPACITY),
            textures,

            fontdue_key_map: HashMap::new(),
            font_key_map: HashMap::new(),
            texture_key_map,

            _should_reset_fontdue_font_key_map: true,
            _should_reset_font_key_map: true,
            _should_reset_texture_key_map: true,
        }
    }

    pub fn insert_bytes(&mut self, name: String, bytes: Vec<u8>) {
        self.bytes.insert(name, bytes);
    }

    pub fn insert_fontdue_font(&mut self, key: FontKey, font: fontdue::Font) {
        self.fontdue_fonts.push(font);
        self.fontdue_key_map
            .insert(key, self.fontdue_fonts.len() - 1);
    }

    pub fn insert_font(&mut self, ctx: &mut Context, key: FontKey, font: Font) -> Result<(), EmeraldError> {
        self.fonts.push(font);
        self.font_key_map
            .insert(key.clone(), self.fonts.len() - 1);

        let mut default_asset_stored_chars = ascii_character_list();
        default_asset_stored_chars.push('.');
        default_asset_stored_chars.push('!');
        default_asset_stored_chars.push('?');

        self.populate_font_cache(ctx, &key, &default_asset_stored_chars, key.1 as u16)
    }

    pub fn insert_texture(&mut self, key: TextureKey, texture: Texture) {
        if let Some(_) = self.get_texture(&key) {
            self.remove_texture(key.clone());
        }

        self.textures.push(texture);
        self.texture_key_map.insert(key, self.textures.len() - 1);
    }

    pub fn get_bytes(&self, name: &String) -> Option<Vec<u8>> {
        if let Some(bytes) = self.bytes.get(name) {
            return Some(bytes.clone());
        }

        None
    }

    pub fn get_fontdue_font(&self, key: &FontKey) -> Option<&fontdue::Font> {
        if let Some(index) = self.fontdue_key_map.get(&key) {
            return self.fontdue_fonts.get(*index);
        }

        None
    }

    pub fn _get_fontdue_font_mut(&mut self, key: &FontKey) -> Option<&mut fontdue::Font> {
        if let Some(index) = self.fontdue_key_map.get(&key) {
            return self.fontdue_fonts.get_mut(*index);
        }

        None
    }

    pub fn get_font(&self, key: &FontKey) -> Option<&Font> {
        if let Some(index) = self.font_key_map.get(&key) {
            return self.fonts.get(*index);
        }

        None
    }

    pub fn get_font_mut(&mut self, key: &FontKey) -> Option<&mut Font> {
        if let Some(index) = self.fontdue_key_map.get(&key) {
            return self.fonts.get_mut(*index);
        }

        None
    }

    pub fn get_texture(&self, key: &TextureKey) -> Option<&Texture> {
        if let Some(index) = self.texture_key_map.get(key) {
            return self.textures.get(*index);
        }

        None
    }

    pub fn _get_texture_mut(&mut self, key: &TextureKey) -> Option<&mut Texture> {
        if let Some(index) = self.texture_key_map.get(key) {
            return self.textures.get_mut(*index);
        }

        None
    }

    pub fn remove_texture(&mut self, key: TextureKey) -> Option<Texture> {
        let mut i = -1;

        if let Some(index) = self.texture_key_map.get(&key) {
            i = *index as _;
        }

        if i >= 0 {
            self.texture_key_map.remove(&key);
            return Some(self.textures.remove(i as _));
        }

        None
    }

    #[inline]
    pub fn update_font_texture(&mut self, mut ctx: &mut Context, key: &FontKey) {
        if let Some(index) = self.font_key_map.get(&key) {
            if let Some(font) = self.fonts.get_mut(*index) {
                if let Some(index) = self.texture_key_map.get(&font.font_texture_key) {
                    if let Some(font_texture) = self.textures.get_mut(*index) {
                        font_texture.update(&mut ctx, &font.font_image);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn populate_font_cache(
        &mut self,
        ctx: &mut Context,
        key: &FontKey,
        characters: &[char],
        size: u16,
    ) -> Result<(), EmeraldError> {
        if let Some(index) = self.font_key_map.get(key) {
            if let Some(_) = self.fonts.get_mut(*index) {
                for character in characters {
                    cache_glyph(ctx, self, &key, *character, size)?;
                }
            }
        }

        Ok(())
    }
}
