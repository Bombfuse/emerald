use std::{any::TypeId, collections::HashMap};

use rapier2d::crossbeam::channel::TryRecvError;

use crate::{
    asset_key::{Asset, AssetId, AssetKey, RefChange, RefChangeChannel},
    EmeraldError,
};

pub(crate) struct AssetStorage {
    asset_uid: AssetId,
    assets: HashMap<AssetId, Asset>,
    asset_references: HashMap<AssetId, isize>,
    ref_change_channel: RefChangeChannel,
}
impl AssetStorage {
    pub fn new() -> Self {
        Self {
            asset_uid: 0,
            assets: HashMap::new(),
            asset_references: HashMap::new(),
            ref_change_channel: RefChangeChannel::default(),
        }
    }

    pub fn count(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    pub fn add(&mut self, asset: Asset, type_id: TypeId) -> Result<AssetKey, EmeraldError> {
        let asset_id = self.asset_uid;
        self.asset_uid += 1;

        self.assets.insert(asset_id, asset);
        self.asset_references.insert(asset_id, 1);

        let key = AssetKey {
            type_id,
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

    /// Consumes all asset reference messages and then
    /// frees all assets that have no references to them.
    pub fn update(&mut self) -> Result<(), EmeraldError> {
        let mut changes_by_asset_id = HashMap::new();

        loop {
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

            self.asset_references.get_mut(&id).map(|count| {
                *count += change_value;

                if *count <= 0 {
                    to_free.push(id);
                }
            });
        }

        for id in to_free {
            self.free_asset(&id)
        }

        Ok(())
    }

    fn free_asset(&mut self, id: &AssetId) {
        self.asset_references.remove(&id);
        self.assets.remove(&id);
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
        let mut asset_storage = AssetStorage::new();
        let expected_value = 10;
        let asset = TestAsset {
            value: expected_value,
        };
        let type_id = asset.type_id();
        let _key = asset_storage.add(Box::new(asset), type_id).unwrap();
        assert_eq!(asset_storage.count(), 1);
    }

    #[test]
    fn auto_drops_assets() {
        let mut asset_storage = AssetStorage::new();
        let expected_value = 10;
        let asset = TestAsset {
            value: expected_value,
        };
        let type_id = asset.type_id();
        let key = asset_storage.add(Box::new(asset), type_id).unwrap();
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
}
