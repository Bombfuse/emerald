use std::{collections::HashMap, io::Cursor, path};

use emerald::{
    assets::asset_engine::AssetEngine, AudioEngine, EmeraldError, Sound, SoundFormat, SoundKey,
    ThreadSafeMixer,
};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};

use crate::audio::mixer::new_mixer;

pub struct DesktopAudioEngine {
    mixers: HashMap<String, ThreadSafeMixer>,
}
impl DesktopAudioEngine {
    pub fn new() -> Self {
        Self {
            mixers: HashMap::new(),
        }
    }
}
impl AudioEngine for DesktopAudioEngine {
    fn initialize(&mut self, _asset_engine: &mut AssetEngine) {}

    fn mixer(
        &mut self,
        mixer_name: &str,
    ) -> Result<&mut emerald::ThreadSafeMixer, emerald::EmeraldError> {
        if !self.mixers.contains_key(mixer_name) {
            self.mixers.insert(mixer_name.into(), new_mixer()?);
        }

        match self.mixers.get_mut(mixer_name) {
            Some(m) => Ok(m),
            None => Err(EmeraldError::new(format!(
                "Unable to get mixer {:?}",
                mixer_name
            ))),
        }
    }

    fn post_update(&mut self) -> Result<(), emerald::EmeraldError> {
        for mixer in self.mixers.values_mut() {
            mixer.post_update()?;
        }
        Ok(())
    }

    fn clear(&mut self) -> Result<(), emerald::EmeraldError> {
        for mixer in self.mixers.values_mut() {
            mixer.clear()?;
        }
        self.mixers = HashMap::new();

        Ok(())
    }

    fn is_sound_loaded(&self, path: &str, asset_engine: &mut AssetEngine) -> bool {
        asset_engine.get_asset_by_label::<Sound>(path).is_some()
    }

    fn load_sound(
        &mut self,
        path: &str,
        bytes: Vec<u8>,
        format: emerald::SoundFormat,
        asset_engine: &mut AssetEngine,
    ) -> Result<emerald::SoundKey, EmeraldError> {
        let static_sound_data = match StaticSoundData::from_cursor(
            Cursor::new(bytes),
            StaticSoundSettings::default(),
        ) {
            Ok(s) => Ok(s),
            Err(e) => Err(EmeraldError::new(
                "Unable to load static sound data from bytes given",
            )),
        }?;
        let asset_key = asset_engine.add_asset_with_label(Box::new(static_sound_data), path)?;
        Ok(SoundKey::new(asset_key, format))
    }

    fn get_sound_key(
        &mut self,
        path: &str,
        asset_engine: &mut AssetEngine,
    ) -> Result<emerald::SoundKey, EmeraldError> {
        asset_engine
            .get_asset_key_by_label::<StaticSoundData>(path)
            .map(|key| Ok(SoundKey::new(key, SoundFormat::Ogg)))
            .unwrap_or(Err(EmeraldError::new(format!(
                "Unable to find sound key for {:?}",
                path
            ))))
    }
}
