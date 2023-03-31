use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{
    asset_key::{Asset, AssetId, AssetKey},
    asset_storage::AssetStorage,
    EmeraldError,
};

const DEFAULT_ASSET_FOLDER: &str = "./assets/";

pub(crate) struct AssetEngine {
    pub(crate) asset_folder_root: String,
    asset_stores: HashMap<TypeId, AssetStorage>,
}
impl AssetEngine {
    pub(crate) fn new() -> Self {
        Self {
            asset_folder_root: DEFAULT_ASSET_FOLDER.to_string(),
            asset_stores: HashMap::new(),
        }
    }

    pub fn get_asset_key_by_label<T: Any>(&self, path: &str) -> Option<AssetKey> {
        let type_id = std::any::TypeId::of::<T>();

        self.asset_stores
            .get(&type_id)
            .map(|store| store.get_asset_key(path))
            .flatten()
    }

    pub fn get_asset_key_by_id<T: Any>(&self, id: &AssetId) -> Option<AssetKey> {
        let type_id = std::any::TypeId::of::<T>();

        self.asset_stores
            .get(&type_id)
            .map(|store| store.get_asset_key_by_id(id))
            .flatten()
    }

    pub fn get_asset<T: Any>(&self, asset_id: &AssetId) -> Option<&T> {
        let type_id = std::any::TypeId::of::<T>();
        self.asset_stores
            .get(&type_id)
            .map(|store| {
                store
                    .get(asset_id)
                    .map(|asset| asset.downcast_ref::<T>())
                    .flatten()
            })
            .flatten()
    }

    pub fn get_asset_mut<T: Any>(&mut self, asset_id: &AssetId) -> Option<&mut T> {
        let type_id = std::any::TypeId::of::<T>();
        self.asset_stores
            .get_mut(&type_id)
            .map(|store| {
                store
                    .get_mut(asset_id)
                    .map(|asset| asset.downcast_mut::<T>())
                    .flatten()
            })
            .flatten()
    }

    pub fn add_asset(&mut self, asset: Asset) -> Result<AssetKey, EmeraldError> {
        self.add_asset_ext(asset, None)
    }

    pub fn add_asset_with_label<T: Into<String>>(
        &mut self,
        asset: Asset,
        path: T,
    ) -> Result<AssetKey, EmeraldError> {
        self.add_asset_ext(asset, Some(path.into()))
    }

    pub fn add_asset_ext(
        &mut self,
        asset: Asset,
        path: Option<String>,
    ) -> Result<AssetKey, EmeraldError> {
        let type_id = (&*asset).type_id();

        if !self.asset_stores.contains_key(&type_id) {
            self.asset_stores
                .insert(type_id, AssetStorage::new(type_id));
        }

        if let Some(asset_store) = self.asset_stores.get_mut(&type_id) {
            return asset_store.add(asset, path);
        }

        Err(EmeraldError::new(format!(
            "No asset store found for TypeId {:?}",
            type_id
        )))
    }

    pub fn read_asset_file(&mut self, relative_path: &str) -> Result<Vec<u8>, EmeraldError> {
        let full_path = self.get_full_asset_path(relative_path);
        read_file(&full_path)
    }

    pub fn get_full_asset_path(&self, path: &str) -> String {
        // If it already contains the correct directory then just return it
        if path.contains(&self.asset_folder_root) {
            return path.to_string();
        }

        let mut full_path = self.asset_folder_root.clone();
        full_path.push_str(path);

        full_path
    }

    pub fn count(&self) -> usize {
        self.asset_stores
            .iter()
            .map(|(_, store)| store.count())
            .sum()
    }

    /// Called after each frame, cleans up unused assets.
    pub fn update(&mut self) -> Result<(), EmeraldError> {
        let mut to_remove = Vec::new();
        for (id, store) in self.asset_stores.iter_mut() {
            store.update()?;

            if store.is_empty() {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            self.asset_stores.remove(&id);
        }

        Ok(())
    }
}
impl Drop for AssetEngine {
    fn drop(&mut self) {
        self.update().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use crate::AssetEngine;
    struct TestAsset {}
    struct TestAsset2 {}

    #[test]
    fn holds_multiple_asset_types() {
        let value1 = TestAsset {};
        let value2 = TestAsset2 {};
        let type_1 = value1.type_id();
        let type_2 = value2.type_id();

        let mut engine = AssetEngine::new();
        engine.add_asset(Box::new(value1)).unwrap();
        engine.add_asset(Box::new(value2)).unwrap();

        assert!(engine.asset_stores.contains_key(&type_1));
        assert!(engine.asset_stores.contains_key(&type_2));
    }

    #[test]
    fn removes_unused_stores() {
        let mut engine = AssetEngine::new();
        let inner = TestAsset {};
        let type_id = inner.type_id();
        let value = Box::new(inner);
        assert_eq!(engine.asset_stores.len(), 0);
        let key = engine.add_asset(value).unwrap();
        assert_eq!(type_id, key.type_id);
        assert_eq!(engine.asset_stores.len(), 1);
        assert!(engine.asset_stores.contains_key(&type_id));

        engine.update().unwrap();
        assert!(engine.asset_stores.contains_key(&type_id));

        drop(key);
        engine.update().unwrap();
        assert!(!engine.asset_stores.contains_key(&type_id));
    }
}

// use crate::font::{Font, FontKey};
// use crate::texture::{Texture, TextureKey};
// use crate::{AssetLoadConfig, EmeraldError, Sound, SoundKey};

// use std::collections::HashMap;
// use std::fs::create_dir;
// use std::path::Path;

// const INITIAL_TEXTURE_STORAGE_CAPACITY: usize = 100;
// const INITIAL_FONT_STORAGE_CAPACITY: usize = 100;

// const DEFAULT_ASSET_FOLDER: &str = "./assets/";

// /// Default to storing user data in the application directory.
// /// Note: This will destroy any user/save files if the game is re-installed.
// const DEFAULT_USER_DATA_FOLDER: &str = "./";

// // const INITIAL_SOUND_STORAGE_CAPACITY: usize = 100;

// /// The AssetEngine stores all Textures, Fonts, and Audio for the game.
// /// It stores the data contiguously, and does caching internally.
// /// Assets can be loaded via the `AssetLoader` and inserted into the AssetEngine.
// /// Assets can be manually removed from the store if memory management becomes a concern.
// pub(crate) struct AssetEngine {
//     pub(crate) load_config: AssetLoadConfig,

//     bytes: HashMap<String, Vec<u8>>,

//     fonts: Vec<Font>,
//     fontdue_fonts: Vec<fontdue::Font>,
//     textures: Vec<Texture>,

//     fontdue_key_map: HashMap<FontKey, usize>,
//     font_key_map: HashMap<FontKey, usize>,
//     pub texture_key_map: HashMap<TextureKey, usize>,

//     pub sound_map: HashMap<SoundKey, Sound>,
//     asset_folder_root: String,
//     user_data_folder_root: String,

//     #[cfg(feature = "hotreload")]
//     pub(crate) file_hot_reload_metadata:
//         HashMap<String, crate::assets::hotreload::HotReloadMetadata>,
// }
// impl AssetEngine {
//     pub fn new(_game_name: String) -> Result<Self, EmeraldError> {
//         let mut texture_key_map = HashMap::new();
//         texture_key_map.insert(TextureKey::default(), 0);
//         let textures = Vec::with_capacity(INITIAL_TEXTURE_STORAGE_CAPACITY);

//         let asset_folder_root = String::from(DEFAULT_ASSET_FOLDER);

//         #[cfg(not(target_os = "windows"))]
//         let user_data_folder_root = String::from(DEFAULT_USER_DATA_FOLDER);

//         #[cfg(target_os = "windows")]
//         let user_data_folder_root =
//             String::from(format!("{}/{}/", get_app_data_directory(), _game_name));

//         #[cfg(not(target_arch = "wasm32"))]
//         if !Path::new(&user_data_folder_root).exists() {
//             create_dir(&user_data_folder_root)?;
//         }

//         Ok(AssetEngine {
//             load_config: Default::default(),
//             bytes: HashMap::new(),
//             fontdue_fonts: Vec::with_capacity(INITIAL_FONT_STORAGE_CAPACITY),
//             fonts: Vec::with_capacity(INITIAL_FONT_STORAGE_CAPACITY),
//             textures,

//             fontdue_key_map: HashMap::new(),
//             font_key_map: HashMap::new(),
//             texture_key_map,

//             sound_map: HashMap::new(),
//             asset_folder_root,
//             user_data_folder_root,

//             #[cfg(feature = "hotreload")]
//             file_hot_reload_metadata: HashMap::new(),
//         })
//     }

//     pub fn set_asset_folder_root(&mut self, root: String) {
//         self.asset_folder_root = root;
//     }

//     pub fn set_user_data_folder_root(&mut self, root: String) {
//         self.user_data_folder_root = root;
//     }

//     pub fn get_asset_folder_root(&mut self) -> String {
//         self.asset_folder_root.clone()
//     }

//     pub fn get_user_data_folder_root(&mut self) -> String {
//         self.user_data_folder_root.clone()
//     }

//     pub fn insert_asset_bytes(
//         &mut self,
//         relative_path: String,
//         bytes: Vec<u8>,
//     ) -> Result<(), EmeraldError> {
//         let path = self.get_full_asset_path(&relative_path);
//         self.bytes.insert(path, bytes);

//         Ok(())
//     }
//     pub fn get_asset_bytes(&mut self, relative_path: &str) -> Option<Vec<u8>> {
//         let full_path = self.get_full_asset_path(relative_path);
//         self.get_bytes(full_path)
//     }
//     pub fn _insert_user_bytes(
//         &mut self,
//         relative_path: String,
//         bytes: Vec<u8>,
//     ) -> Result<(), EmeraldError> {
//         let path = self.get_full_user_data_path(&relative_path);
//         self.bytes.insert(path, bytes);

//         Ok(())
//     }
//     pub fn get_user_bytes(&mut self, relative_path: &str) -> Option<Vec<u8>> {
//         let full_path = self.get_full_user_data_path(relative_path);
//         self.get_bytes(full_path)
//     }
//     pub fn read_user_file(&mut self, relative_path: &str) -> Result<Vec<u8>, EmeraldError> {
//         let full_path = self.get_full_user_data_path(relative_path);
//         read_file(&full_path)
//     }

//     fn get_bytes(&mut self, path: String) -> Option<Vec<u8>> {
//         if let Some(bytes) = self.bytes.get(&path) {
//             return Some(bytes.clone());
//         }

//         None
//     }

//     pub fn insert_fontdue_font(&mut self, key: FontKey, font: fontdue::Font) {
//         self.fontdue_fonts.push(font);
//         self.fontdue_key_map
//             .insert(key, self.fontdue_fonts.len() - 1);
//     }

//     pub fn insert_font(&mut self, key: FontKey, font: Font) -> Result<(), EmeraldError> {
//         self.fonts.push(font);
//         self.font_key_map.insert(key, self.fonts.len() - 1);

//         Ok(())
//     }

//     pub fn insert_texture(&mut self, key: TextureKey, texture: Texture) {
//         if self.get_texture(&key).is_some() {
//             self.remove_texture(key.clone());
//         }

//         self.textures.push(texture);
//         self.texture_key_map
//             .insert(key.clone(), self.textures.len() - 1);

//         #[cfg(feature = "hotreload")]
//         crate::assets::hotreload::on_insert_texture(
//             self,
//             &self.get_full_asset_path(&key.get_name()),
//         )
//     }

//     pub fn get_full_user_data_path(&self, path: &str) -> String {
//         // If it already contains the correct directory then just return it
//         if path.contains(&self.user_data_folder_root) {
//             return path.to_string();
//         }

//         let mut full_path = self.user_data_folder_root.clone();
//         full_path.push_str(path);

//         full_path
//     }

//     pub fn get_fontdue_font(&self, key: &FontKey) -> Option<&fontdue::Font> {
//         if let Some(index) = self.fontdue_key_map.get(key) {
//             return self.fontdue_fonts.get(*index);
//         }

//         None
//     }

//     pub fn _get_fontdue_font_mut(&mut self, key: &FontKey) -> Option<&mut fontdue::Font> {
//         if let Some(index) = self.fontdue_key_map.get(key) {
//             return self.fontdue_fonts.get_mut(*index);
//         }

//         None
//     }

//     pub fn get_font(&self, key: &FontKey) -> Option<&Font> {
//         if let Some(index) = self.font_key_map.get(key) {
//             return self.fonts.get(*index);
//         }

//         None
//     }

//     pub fn get_font_mut(&mut self, key: &FontKey) -> Option<&mut Font> {
//         if let Some(index) = self.fontdue_key_map.get(key) {
//             return self.fonts.get_mut(*index);
//         }

//         None
//     }

//     pub fn get_texture(&self, key: &TextureKey) -> Option<&Texture> {
//         if let Some(index) = self.texture_key_map.get(key) {
//             return self.textures.get(*index);
//         }

//         None
//     }

//     pub fn _get_texture_mut(&mut self, key: &TextureKey) -> Option<&mut Texture> {
//         if let Some(index) = self.texture_key_map.get(key) {
//             return self.textures.get_mut(*index);
//         }

//         None
//     }

//     pub fn remove_texture(&mut self, key: TextureKey) -> Option<Texture> {
//         let mut i: i32 = -1;

//         if let Some(index) = self.texture_key_map.get(&key) {
//             i = *index as _;
//         }

//         if i >= 0 {
//             // No need to reset map if only the end texture is removed.
//             let reset_map = (i as usize) != self.textures.len();
//             self.texture_key_map.remove(&key);
//             let texture = self.textures.remove(i as _);

//             if reset_map {
//                 self.update_texture_key_map();
//             }

//             return Some(texture);
//         }

//         None
//     }

//     #[inline]
//     pub fn update_texture_key_map(&mut self) {
//         self.texture_key_map = HashMap::with_capacity(self.textures.len());

//         let mut i = 0;

//         for texture in &self.textures {
//             self.texture_key_map.insert(texture.key.clone(), i);
//             i += 1;
//         }
//     }

//     #[inline]
//     pub fn contains_sound(&self, key: &SoundKey) -> bool {
//         self.sound_map.contains_key(key)
//     }

//     #[inline]
//     pub fn insert_sound(&mut self, key: SoundKey, sound: Sound) {
//         self.sound_map.insert(key, sound);
//     }
// }

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
        let slice = unsafe { std::slice::from_raw_parts(data.content, data.content_length as _) };
        let response = slice.iter().map(|c| *c as _).collect::<Vec<_>>();
        Ok(response)
    } else {
        Err(EmeraldError::new(format!(
            "Unable to load asset `{}`",
            path
        )))
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn read_file(path: &str) -> Result<Vec<u8>, EmeraldError> {
    use std::fs::File;
    use std::io::Read;

    let current_dir = std::env::current_dir()?;
    let file_path = current_dir.join(path);
    let file_path = file_path.into_os_string().into_string()?;

    let mut contents = vec![];
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            return Err(EmeraldError::new(format!(
                "Error loading file {:?}: {:?}",
                path, e
            )))
        }
    };
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

// // Source
// // https://github.com/dirs-dev/dirs-sys-rs/blob/main/src/lib.rs
// #[cfg(target_os = "windows")]
// fn get_app_data_directory() -> String {
//     use std::ffi::OsString;
//     use std::os::windows::ffi::OsStringExt;
//     use std::path::PathBuf;
//     use std::ptr;
//     use std::slice;

//     use winapi::shared::winerror;
//     use winapi::um::{combaseapi, knownfolders, shlobj, shtypes, winbase, winnt};

//     pub fn known_folder(folder_id: shtypes::REFKNOWNFOLDERID) -> Option<PathBuf> {
//         unsafe {
//             let mut path_ptr: winnt::PWSTR = ptr::null_mut();
//             let result = shlobj::SHGetKnownFolderPath(folder_id, 0, ptr::null_mut(), &mut path_ptr);
//             if result == winerror::S_OK {
//                 let len = winbase::lstrlenW(path_ptr) as usize;
//                 let path = slice::from_raw_parts(path_ptr, len);
//                 let ostr: OsString = OsStringExt::from_wide(path);
//                 combaseapi::CoTaskMemFree(path_ptr as *mut winapi::ctypes::c_void);
//                 Some(PathBuf::from(ostr))
//             } else {
//                 combaseapi::CoTaskMemFree(path_ptr as *mut winapi::ctypes::c_void);
//                 None
//             }
//         }
//     }

//     if let Some(folder) = known_folder(&knownfolders::FOLDERID_RoamingAppData) {
//         if let Some(s) = folder.to_str() {
//             return s.to_string();
//         }
//     }

//     String::from(DEFAULT_USER_DATA_FOLDER)
// }
