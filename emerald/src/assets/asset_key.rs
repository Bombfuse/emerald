use std::any::{Any, TypeId};

use rapier2d::crossbeam::channel::{Receiver, Sender};

pub type AssetId = usize;

pub type Asset = Box<dyn Any>;

#[derive(Debug)]
pub struct AssetKey {
    pub(crate) type_id: TypeId,
    pub(crate) asset_id: AssetId,
    pub(crate) ref_sender: Sender<RefChange>,
}
impl AssetKey {
    pub(crate) fn new(asset_id: AssetId, type_id: TypeId, ref_sender: Sender<RefChange>) -> Self {
        ref_sender.send(RefChange::Increment(asset_id)).unwrap();

        Self {
            type_id,
            asset_id,
            ref_sender,
        }
    }

    pub fn asset_id(&self) -> AssetId {
        self.asset_id
    }
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}
impl PartialEq for AssetKey {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id && self.asset_id == other.asset_id
    }
}
impl Clone for AssetKey {
    fn clone(&self) -> Self {
        self.ref_sender
            .send(RefChange::Increment(self.asset_id))
            .unwrap();

        Self {
            type_id: self.type_id.clone(),
            asset_id: self.asset_id.clone(),
            ref_sender: self.ref_sender.clone(),
        }
    }
}
impl Drop for AssetKey {
    fn drop(&mut self) {
        match self.ref_sender
            .send(RefChange::Decrement(self.asset_id))
            {
                Ok(_) =>{},
                Err(e) => {
                    println!("{:?}", format!(
                        "Fatal Error: Failed to drop asset {:?}",
                        (self.type_id, self.asset_id, e.to_string())
                    ));
                }
            }
    }
}

pub(crate) enum RefChange {
    Increment(AssetId),
    Decrement(AssetId),
}

#[derive(Clone)]
pub(crate) struct RefChangeChannel {
    pub sender: Sender<RefChange>,
    pub receiver: Receiver<RefChange>,
}
impl Default for RefChangeChannel {
    fn default() -> Self {
        let (sender, receiver) = crate::crossbeam::channel::unbounded();
        RefChangeChannel { sender, receiver }
    }
}
