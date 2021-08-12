use crate::rendering::*;
use crate::{EmeraldError, Sound, SoundKey};

use miniquad::Context;
use std::collections::HashMap;



const INITIAL_TEXTURE_STORAGE_CAPACITY: usize = 100;
const INITIAL_FONT_STORAGE_CAPACITY: usize = 100;

const DEFAULT_ASSET_FOLDER: &str = "./assets/";

/// Default to storing user data in the application directory.
/// Note: This will destroy any user/save files if the game is re-installed.
const DEFAULT_USER_DATA_FOLDER: &str = "./";




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
    pub texture_key_map: HashMap<TextureKey, usize>,

    pub sound_map: HashMap<SoundKey, Sound>,
    asset_folder_root: String,
    user_data_folder_root: String,

    #[cfg(feature="hotreload")]
    pub(crate) file_hot_reload_metadata: HashMap<String, crate::assets::hotreload::HotReloadMetadata>,
}
impl AssetStore {
    pub fn new(ctx: &mut Context) -> Self {
        let mut texture_key_map = HashMap::new();
        let default_texture = Texture::default(ctx).unwrap();
        texture_key_map.insert(TextureKey::default(), 0);

        let mut textures = Vec::with_capacity(INITIAL_TEXTURE_STORAGE_CAPACITY);
        textures.push(default_texture);

        let asset_folder_root = String::from(DEFAULT_ASSET_FOLDER);
        let user_data_folder_root = String::from(DEFAULT_USER_DATA_FOLDER);

        AssetStore {
            bytes: HashMap::new(),
            fontdue_fonts: Vec::with_capacity(INITIAL_FONT_STORAGE_CAPACITY),
            fonts: Vec::with_capacity(INITIAL_FONT_STORAGE_CAPACITY),
            textures,

            fontdue_key_map: HashMap::new(),
            font_key_map: HashMap::new(),
            texture_key_map,

            sound_map: HashMap::new(),
            asset_folder_root,
            user_data_folder_root,
            
            #[cfg(feature="hotreload")]
            file_hot_reload_metadata: HashMap::new(),
        }
    }

    pub fn set_asset_folder_root(&mut self, root: String) {
        self.asset_folder_root = root;
    }

    pub fn set_user_data_folder_root(&mut self, root: String) {
        self.user_data_folder_root = root;
    }

    pub fn get_asset_folder_root(&mut self) -> String {
        self.asset_folder_root.clone()
    }

    pub fn get_user_data_folder_root(&mut self) -> String {
        self.user_data_folder_root.clone()
    }

    pub fn insert_asset_bytes(&mut self, relative_path: String, bytes: Vec<u8>) -> Result<(), EmeraldError> {
        let path = self.get_full_asset_path(&relative_path);
        self.bytes.insert(path, bytes);

        Ok(())
    }
    pub fn get_asset_bytes(&mut self, relative_path: &String) -> Option<Vec<u8>> {
        let full_path = self.get_full_asset_path(relative_path);
        self.get_bytes(full_path)
    }

    pub fn read_asset_file(&mut self, relative_path: &String) -> Result<Vec<u8>, EmeraldError> {
        let full_path = self.get_full_asset_path(relative_path);
        read_file(&full_path)
    }

    pub fn _insert_user_bytes(&mut self, relative_path: String, bytes: Vec<u8>) -> Result<(), EmeraldError> {
        let path = self.get_full_user_data_path(&relative_path);
        self.bytes.insert(path, bytes);

        Ok(())
    }
    pub fn get_user_bytes(&mut self, relative_path: &String) -> Option<Vec<u8>> {
        let full_path = self.get_full_user_data_path(relative_path);
        self.get_bytes(full_path)
    }
    pub fn read_user_file(&mut self, relative_path: &String) -> Result<Vec<u8>, EmeraldError> {
        let full_path = self.get_full_user_data_path(relative_path);
        read_file(&full_path)
    }

    fn get_bytes(&mut self, path: String) -> Option<Vec<u8>> {
        if let Some(bytes) = self.bytes.get(&path) {
            return Some(bytes.clone());
        }

        None
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

    pub fn insert_texture(&mut self, key: TextureKey, texture: Texture) {
        if let Some(_) = self.get_texture(&key) {
            self.remove_texture(key.clone());

        }

        self.textures.push(texture);
        self.texture_key_map.insert(key.clone(), self.textures.len() - 1);

        #[cfg(feature="hotreload")]
        crate::assets::hotreload::on_insert_texture(self, self.get_full_asset_path(&key.get_name()))
    }

    pub fn get_full_asset_path(&self, path: &String) -> String {
        // If it already contains the correct directory then just return it
        if path.contains(&self.asset_folder_root) {
            return path.clone();

        }

        let mut full_path = self.asset_folder_root.clone();
        full_path.push_str(path);

        full_path
    }


    pub fn get_full_user_data_path(&self, path: &String) -> String {
        // If it already contains the correct directory then just return it
        if path.contains(&self.user_data_folder_root) {
            return path.clone();
        }

        let mut full_path = self.user_data_folder_root.clone();
        full_path.push_str(path);

        full_path
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

    pub fn remove_texture(&mut self, key: TextureKey) -> Option<Texture> {
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

    #[inline]
    pub fn contains_sound(&self, key: &SoundKey) -> bool {
        self.sound_map.contains_key(key)
    }

    #[inline]
    pub fn insert_sound(&mut self, key: SoundKey, sound: Sound) {
        self.sound_map.insert(key, sound);
    }
}


#[cfg(target_arch = "wasm32")]
fn read_file(path: &str) -> Result<Vec<u8>, EmeraldError> {
    Err(EmeraldError::new(format!(
        "Unable to get bytes for {}",
        path
    )))
}

#[cfg(target_os = "android")]
fn read_file(path: &str) -> Result<Vec<u8>, EmeraldError> {
    // Based on https://github.com/not-fl3/miniquad/blob/4be5328760ff356494caf59cc853bcb395bce5d2/src/fs.rs#L38-L53

    let filename = std::ffi::CString::new(path).unwrap();

    let mut data: sapp_android::android_asset = unsafe { std::mem::zeroed() };

    unsafe { sapp_android::sapp_load_asset(filename.as_ptr(), &mut data as _) };

    if data.content.is_null() == false {
        let slice =
            unsafe { std::slice::from_raw_parts(data.content, data.content_length as _) };
        let response = slice.iter().map(|c| *c as _).collect::<Vec<_>>();
        Ok(response)
    } else {
        Err(EmeraldError::new(format!("Unable to load asset `{}`", path)))
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn read_file(path: &str) -> Result<Vec<u8>, EmeraldError> {
    use std::fs::File;
    use std::io::Read;

    let current_dir = std::env::current_dir()?;
    let path = current_dir.join(path);
    let path = path.into_os_string().into_string()?;

    let mut contents = vec![];
    let mut file = File::open(path)?;
    file.read_to_end(&mut contents)?;
    Ok(contents)
}
