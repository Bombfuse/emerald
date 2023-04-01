use std::collections::HashMap;

use crate::{Emerald, EmeraldError, SoundInstanceId, SoundKey};

/// Holds sound keys and can play them given Emerald.
/// Mostly useful for preloading the sounds needed by Entities, or the World.
pub struct SoundPlayer {
    keys: HashMap<String, SoundKey>,
    mixer: String,
}
impl SoundPlayer {
    pub fn new<T: Into<String>>(mixer: T) -> Self {
        Self {
            mixer: mixer.into(),
            keys: HashMap::new(),
        }
    }

    pub fn add_sound(&mut self, label: &str, key: SoundKey) {
        self.keys.insert(label.to_string(), key);
    }

    pub fn play(&self, emd: &mut Emerald, label: &str) -> Result<SoundInstanceId, EmeraldError> {
        if let Some(key) = self.keys.get(label) {
            let id = emd.audio().mixer(&self.mixer)?.play(key)?;
            return Ok(id);
        }

        Err(EmeraldError::new(format!(
            "{:?} is not loaded into the Sound Player",
            label
        )))
    }
}
