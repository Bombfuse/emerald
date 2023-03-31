use std::{any::TypeId, collections::HashMap};

use rapier2d::crossbeam::channel::TryRecvError;

use crate::{
    asset_key::{Asset, AssetId, AssetKey, RefChange, RefChangeChannel},
    EmeraldError,
};

pub(crate) struct AssetStorage {
    asset_type_id: TypeId,
    asset_uid: AssetId,
    assets: HashMap<AssetId, Asset>,
    asset_references: HashMap<AssetId, isize>,
    asset_paths: HashMap<AssetId, String>,
    label_asset_ids: HashMap<String, AssetId>,
    ref_change_channel: RefChangeChannel,
}
impl AssetStorage {
    pub fn new(asset_type_id: TypeId) -> Self {
        Self {
            asset_type_id,
            asset_uid: 0,
            assets: HashMap::new(),
            asset_references: HashMap::new(),
            asset_paths: HashMap::new(),
            label_asset_ids: HashMap::new(),
            ref_change_channel: RefChangeChannel::default(),
        }
    }

    pub fn count(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    pub fn add<T: Into<String>>(
        &mut self,
        asset: Asset,
        file_path: Option<T>,
    ) -> Result<AssetKey, EmeraldError> {
        let path: Option<String> = file_path.map(|f| f.into());
        if let Some(path) = &path {
            if let Some(asset_id) = self.get_asset_id(path) {
                return self.overwrite_asset(asset, asset_id);
            }
        }

        return self.add_new_asset(asset, path);
    }

    fn overwrite_asset(
        &mut self,
        asset: Asset,
        asset_id: AssetId,
    ) -> Result<AssetKey, EmeraldError> {
        self.assets.insert(asset_id, asset);

        let key = AssetKey {
            type_id: self.asset_type_id,
            asset_id,
            ref_sender: self.ref_change_channel.sender.clone(),
        };

        Ok(key)
    }

    fn add_new_asset(
        &mut self,
        asset: Asset,
        path: Option<String>,
    ) -> Result<AssetKey, EmeraldError> {
        let asset_id = self.asset_uid;
        self.asset_uid += 1;

        self.assets.insert(asset_id, asset);
        self.asset_references.insert(asset_id, 1);
        path.map(|path| self.set_asset_label(asset_id, path));

        let key = AssetKey {
            type_id: self.asset_type_id,
            asset_id,
            ref_sender: self.ref_change_channel.sender.clone(),
        };

        Ok(key)
    }

    pub fn get(&self, id: &AssetId) -> Option<&Asset> {
        self.assets.get(id)
    }

    pub fn get_mut(&mut self, id: &AssetId) -> Option<&mut Asset> {
        self.assets.get_mut(id)
    }

    pub fn get_by_label(&self, label: &str) -> Option<&Asset> {
        self.label_asset_ids
            .get(label)
            .map(|id| self.get(id))
            .flatten()
    }

    pub fn get_mut_by_label(&mut self, label: &str) -> Option<&mut Asset> {
        if let Some(id) = self.label_asset_ids.get(label).map(|id| *id) {
            return self.get_mut(&id);
        }

        None
    }

    pub fn get_asset_key(&self, path: &str) -> Option<AssetKey> {
        self.get_asset_id(path).map(|asset_id| {
            AssetKey::new(
                asset_id,
                self.asset_type_id,
                self.ref_change_channel.sender.clone(),
            )
        })
    }

    pub fn get_asset_key_by_id(&self, id: &AssetId) -> Option<AssetKey> {
        if self.assets.contains_key(id) {
            Some(AssetKey::new(
                *id,
                self.asset_type_id,
                self.ref_change_channel.sender.clone(),
            ))
        } else {
            None
        }
    }

    pub fn get_asset_id(&self, path: &str) -> Option<AssetId> {
        self.label_asset_ids.get(path).map(|id| id.clone())
    }

    fn add_ref_count_by_asset_id(&mut self, asset_id: AssetId, amount: isize) -> isize {
        if !self.asset_references.contains_key(&asset_id) {
            self.asset_references.insert(asset_id, 0);
        }

        self.asset_references
            .get_mut(&asset_id)
            .map(|count| {
                *count += amount;
                count.clone()
            })
            .unwrap()
    }

    /// Consumes all asset reference messages and then
    /// frees all assets that have no references to them.
    pub fn update(&mut self) -> Result<(), EmeraldError> {
        let mut changes_by_asset_id = HashMap::new();

        loop {
            if self.ref_change_channel.receiver.is_empty() {
                break;
            }

            let ref_change = match self.ref_change_channel.receiver.try_recv() {
                Ok(message) => message,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Err(EmeraldError::new("")),
            };

            match ref_change {
                RefChange::Increment(id) => increment_by_asset_id(&mut changes_by_asset_id, id),
                RefChange::Decrement(id) => decrement_by_asset_id(&mut changes_by_asset_id, id),
            };
        }

        let mut to_free = Vec::new();
        for (id, change_value) in changes_by_asset_id {
            if !self.asset_references.contains_key(&id) {
                self.asset_references.insert(id, 0);
            }

            if self.add_ref_count_by_asset_id(id, change_value) <= 0 {
                to_free.push(id);
            }
        }

        for id in to_free {
            self.free_asset(&id)
        }
        Ok(())
    }

    fn set_asset_label(&mut self, asset_id: AssetId, path: String) {
        self.asset_paths.insert(asset_id, path.clone());
        self.label_asset_ids.insert(path, asset_id);
    }

    fn free_asset(&mut self, id: &AssetId) {
        self.asset_references.remove(&id);
        self.assets.remove(&id);

        self.asset_paths
            .remove(&id)
            .map(|path| self.label_asset_ids.remove(&path));
    }
}

fn add_by_asset_id(changes_by_asset_id: &mut HashMap<AssetId, isize>, id: AssetId, amount: isize) {
    if !changes_by_asset_id.contains_key(&id) {
        changes_by_asset_id.insert(id, 0);
    }

    changes_by_asset_id
        .get_mut(&id)
        .map(|value| *value += amount);
}

fn decrement_by_asset_id(changes_by_asset_id: &mut HashMap<AssetId, isize>, id: AssetId) {
    add_by_asset_id(changes_by_asset_id, id, -1)
}
fn increment_by_asset_id(changes_by_asset_id: &mut HashMap<AssetId, isize>, id: AssetId) {
    add_by_asset_id(changes_by_asset_id, id, 1)
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::AssetStorage;

    struct TestAsset {
        pub value: usize,
    }

    #[test]
    fn asset_counts() {
        let expected_value = 10;
        let asset = TestAsset {
            value: expected_value,
        };
        let type_id = asset.type_id();
        let mut asset_storage = AssetStorage::new(type_id);
        let _key = asset_storage.add(Box::new(asset), Some("test")).unwrap();
        assert_eq!(asset_storage.count(), 1);
    }

    #[test]
    fn auto_drops_assets() {
        let expected_value = 10;
        let asset = TestAsset {
            value: expected_value,
        };
        let type_id = asset.type_id();
        let mut asset_storage = AssetStorage::new(type_id);
        let key = asset_storage.add(Box::new(asset), Some("test")).unwrap();
        asset_storage.update().unwrap();
        assert_eq!(asset_storage.count(), 1);
        assert_eq!(
            *asset_storage.asset_references.get(&key.asset_id).unwrap(),
            1
        );

        let key2 = key.clone();
        asset_storage.update().unwrap();
        assert_eq!(
            *asset_storage.asset_references.get(&key.asset_id).unwrap(),
            2
        );

        drop(key);
        asset_storage.update().unwrap();
        assert_eq!(asset_storage.count(), 1);

        drop(key2);
        asset_storage.update().unwrap();
        assert_eq!(asset_storage.count(), 0);
    }

    #[test]
    fn get_asset_key_bumps_up_ref_counter() {
        let expected_value = 10;
        let asset = TestAsset {
            value: expected_value,
        };
        let type_id = asset.type_id();
        let mut asset_storage = AssetStorage::new(type_id);
        let key = asset_storage.add(Box::new(asset), Some("test")).unwrap();
        asset_storage.update().unwrap();
        assert_eq!(
            *asset_storage.asset_references.get(&key.asset_id).unwrap(),
            1
        );

        let key2 = asset_storage.get_asset_key("test").unwrap();
        asset_storage.update().unwrap();
        asset_storage.update().unwrap();
        assert_eq!(
            *asset_storage.asset_references.get(&key.asset_id).unwrap(),
            2
        );

        drop(key);
        asset_storage.update().unwrap();
        assert_eq!(asset_storage.count(), 1);

        drop(key2);
        asset_storage.update().unwrap();
        assert_eq!(asset_storage.count(), 0);
    }
}
