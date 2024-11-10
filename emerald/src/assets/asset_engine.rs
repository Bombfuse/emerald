use core::any::{Any, TypeId};

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::HashMap;
use hecs::Component;
use serde::de::DeserializeOwned;

use crate::{
    asset_key::{Asset, AssetId, AssetKey},
    asset_storage::AssetStorage,
    AssetLoadConfig, AssetLoadContext, EmeraldError, OnAssetLoadCallback, Sound,
};

const DEFAULT_ASSET_FOLDER: &str = "./assets/";
const DEFAULT_USER_DATA_FOLDER: &str = "./";

pub struct AssetEngine {
    pub(crate) user_data_folder_root: String,
    pub(crate) asset_folder_root: String,
    pub(crate) load_config: AssetLoadConfig,
    pub(crate) on_asset_load_callback: Option<OnAssetLoadCallback>,
    asset_stores: HashMap<TypeId, AssetStorage>,
}
impl AssetEngine {
    pub fn new() -> Self {
        Self {
            user_data_folder_root: DEFAULT_USER_DATA_FOLDER.to_string(),
            asset_folder_root: DEFAULT_ASSET_FOLDER.to_string(),
            asset_stores: HashMap::new(),
            load_config: AssetLoadConfig::default(),
            on_asset_load_callback: None,
        }
    }

    pub fn get_asset_by_label<T: Any>(&self, label: &str) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        self.asset_stores
            .get(&type_id)
            .map(|store| store.get_by_label(label))
            .flatten()
            .map(|asset| asset.downcast_ref())
            .flatten()
    }

    pub fn get_asset_mut_by_label<T: Any>(&mut self, label: &str) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();

        self.asset_stores
            .get_mut(&type_id)
            .map(|store| store.get_mut_by_label(label))
            .flatten()
            .map(|asset| asset.downcast_mut())
            .flatten()
    }

    pub fn get_asset_key_by_label<T: Any>(&self, path: &str) -> Option<AssetKey> {
        let type_id = TypeId::of::<T>();

        self.asset_stores
            .get(&type_id)
            .map(|store| store.get_asset_key(path))
            .flatten()
    }

    pub fn get_asset_key_by_id<T: Any>(&self, id: &AssetId) -> Option<AssetKey> {
        let type_id = TypeId::of::<T>();

        self.asset_stores
            .get(&type_id)
            .map(|store| store.get_asset_key_by_id(id))
            .flatten()
    }

    pub fn get_asset<T: Any>(&self, asset_id: &AssetId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
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
        let type_id = TypeId::of::<T>();
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
        todo!("use file reader")
        // read_file(&full_path).map(|bytes| {
        //     if let Some(callback) = &self.on_asset_load_callback {
        //         let context = AssetLoadContext { path: &full_path };
        //         (callback)(context)
        //     }

        //     bytes
        // })
    }

    pub fn read_user_file(&mut self, relative_path: &str) -> Result<Vec<u8>, EmeraldError> {
        let full_path = self.get_full_user_data_path(relative_path);
        todo!("use file reader")
    }

    pub fn get_full_user_data_path(&self, path: &str) -> String {
        // If it already contains the correct directory then just return it
        if path.contains(&self.user_data_folder_root) {
            return path.to_string();
        }

        let mut full_path = self.user_data_folder_root.clone();
        full_path.push_str(path);

        full_path
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

    pub fn total_count(&self) -> usize {
        self.asset_stores
            .iter()
            .map(|(_, store)| store.count())
            .sum()
    }

    pub fn count<T: Any>(&self) -> usize {
        let type_id = TypeId::of::<T>();

        self.asset_stores
            .get(&type_id)
            .map(|store| store.count())
            .unwrap_or(0)
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
