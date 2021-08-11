use crate::rendering::*;
use crate::{EmeraldError, Sound, SoundKey};

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
    pub bytes: HashMap<String, Vec<u8>>,

    pub fonts: Vec<Font>,
    pub fontdue_fonts: Vec<fontdue::Font>,
    pub textures: Vec<Texture>,

    pub fontdue_key_map: HashMap<FontKey, usize>,
    pub font_key_map: HashMap<FontKey, usize>,
    pub texture_key_map: HashMap<TextureKey, usize>,

    pub sound_map: HashMap<SoundKey, Sound>,
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

            sound_map: HashMap::new(),
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

    pub fn insert_font(
        &mut self,
        _ctx: &mut Context,
        key: FontKey,
        font: Font,
    ) -> Result<(), EmeraldError> {
        self.fonts.push(font);
        self.font_key_map.insert(key, self.fonts.len() - 1);

        Ok(())
    }

    pub fn insert_texture(&mut self, ctx: &mut Context, key: TextureKey, texture: Texture) {
        if self.get_texture(&key).is_some() {
            self.remove_texture(ctx, key.clone());
        }

        self.textures.push(texture);
        self.texture_key_map.insert(key, self.textures.len() - 1);
    }

    pub fn get_bytes(&self, name: &str) -> Option<Vec<u8>> {
        if let Some(bytes) = self.bytes.get(name) {
            return Some(bytes.clone());
        }

        None
    }

    pub fn get_fontdue_font(&self, key: &FontKey) -> Option<&fontdue::Font> {
        if let Some(index) = self.fontdue_key_map.get(key) {
            return self.fontdue_fonts.get(*index);
        }

        None
    }

    pub fn _get_fontdue_font_mut(&mut self, key: &FontKey) -> Option<&mut fontdue::Font> {
        if let Some(index) = self.fontdue_key_map.get(key) {
            return self.fontdue_fonts.get_mut(*index);
        }

        None
    }

    pub fn get_font(&self, key: &FontKey) -> Option<&Font> {
        if let Some(index) = self.font_key_map.get(key) {
            return self.fonts.get(*index);
        }

        None
    }

    pub fn get_font_mut(&mut self, key: &FontKey) -> Option<&mut Font> {
        if let Some(index) = self.fontdue_key_map.get(key) {
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

    pub fn remove_texture(&mut self, _ctx: &mut Context, key: TextureKey) -> Option<Texture> {
        let mut i: i32 = -1;

        if let Some(index) = self.texture_key_map.get(&key) {
            i = *index as _;
        }

        if i >= 0 {
            // No need to reset map if only the end texture is removed.
            let reset_map = (i as usize) != self.textures.len();
            self.texture_key_map.remove(&key);
            let texture = self.textures.remove(i as _);
            texture.inner.delete();

            if reset_map {
                self.update_texture_key_map();
            }

            return Some(texture);
        }

        None
    }

    #[inline]
    pub fn update_texture_key_map(&mut self) {
        self.texture_key_map = HashMap::with_capacity(self.textures.len());

        let mut i = 0;

        for texture in &self.textures {
            self.texture_key_map.insert(texture.key.clone(), i);
            i += 1;
        }
    }

    #[inline]
    pub fn update_font_texture(&mut self, mut ctx: &mut Context, key: &FontKey) {
        if let Some(index) = self.font_key_map.get(key) {
            if let Some(font) = self.fonts.get_mut(*index) {
                if let Some(index) = self.texture_key_map.get(&font.font_texture_key) {
                    if let Some(font_texture) = self.textures.get_mut(*index) {
                        font_texture.update(&mut ctx, &font.font_image);
                    }
                }
            }
        }
    }
}
