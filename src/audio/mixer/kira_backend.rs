use std::collections::HashMap;

use kira::{
    instance::{handle::InstanceHandle, InstanceSettings, InstanceState},
    manager::{AudioManager, AudioManagerSettings},
    sound::{handle::SoundHandle, SoundId},
};

use crate::{AssetStore, EmeraldError, Mixer, SoundInstanceId, SoundKey};

pub(crate) struct KiraMixer {
    inner: AudioManager,
    sound_handles: HashMap<SoundId, SoundHandle>,
    instances: HashMap<SoundInstanceId, InstanceHandle>,
    volume: f64,
    sound_instance_uid: usize,
}
impl KiraMixer {
    pub fn new() -> Result<Box<dyn Mixer>, EmeraldError> {
        Ok(Box::new(KiraMixer {
            inner: AudioManager::new(AudioManagerSettings {
                num_sounds: 1000,
                num_instances: 1000,
                ..Default::default()
            })?,
            sound_handles: HashMap::new(),
            instances: HashMap::new(),
            volume: 1.0,
            sound_instance_uid: 0,
        }))
    }
}

impl Mixer for KiraMixer {
    /// Plays the given sound data once.
    /// Applies the volume of the mixer to the sound.
    fn play(
        &mut self,
        key: SoundKey,
        asset_store: &mut AssetStore,
    ) -> Result<SoundInstanceId, EmeraldError> {
        if let Some(sound) = asset_store.sound_map.get(&key) {
            let id = sound.inner.id();

            let mut sound_handle = if let Some(sound_handle) = self.sound_handles.get_mut(&id) {
                sound_handle.clone()
            } else {
                let handle = self.inner.add_sound(sound.inner.clone())?;
                self.sound_handles.insert(id, handle.clone());

                handle
            };

            let instance_handle = sound_handle.play(InstanceSettings::new().volume(self.volume))?;

            let id = SoundInstanceId::new(self.sound_instance_uid);
            self.sound_instance_uid += 1;
            self.instances.insert(id, instance_handle);

            return Ok(id);
        }

        Err(EmeraldError::new(format!(
            "Sound for {:?} is not loaded in the asset store.",
            key
        )))
    }

    fn play_and_loop(
        &mut self,
        key: SoundKey,
        asset_store: &mut AssetStore,
    ) -> Result<SoundInstanceId, EmeraldError> {
        if let Some(sound) = asset_store.sound_map.get(&key) {
            let id = sound.inner.id();

            let mut sound_handle = if let Some(sound_handle) = self.sound_handles.get_mut(&id) {
                sound_handle.clone()
            } else {
                let handle = self.inner.add_sound(sound.inner.clone())?;
                self.sound_handles.insert(id, handle.clone());

                handle
            };

            let instance_handle = sound_handle.play(
                InstanceSettings::new()
                    .volume(self.volume)
                    .loop_start(kira::instance::InstanceLoopStart::Custom(0.0)),
            )?;

            let id = SoundInstanceId::new(self.sound_instance_uid);
            self.sound_instance_uid += 1;
            self.instances.insert(id, instance_handle);

            return Ok(id);
        }

        Err(EmeraldError::new(format!(
            "Sound for {:?} is not loaded in the asset store.",
            key
        )))
    }

    fn get_volume(&self) -> Result<f32, EmeraldError> {
        Ok(self.volume as f32)
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), EmeraldError> {
        self.volume = volume as f64;

        for instance in self.instances.values_mut() {
            instance.set_volume(self.volume)?;
        }

        Ok(())
    }

    fn set_instance_volume(
        &mut self,
        snd_instance_id: SoundInstanceId,
        volume: f32,
    ) -> Result<(), EmeraldError> {
        if let Some(instance) = self.instances.get_mut(&snd_instance_id) {
            instance.set_volume(volume as f64)?;
        }

        Ok(())
    }

    /// Get the ids of all instances in the mixer.
    fn get_instances(&self) -> Result<Vec<SoundInstanceId>, EmeraldError> {
        let mut instances = Vec::new();

        for key in self.instances.keys() {
            instances.push(*key);
        }

        Ok(instances)
    }

    /// Stop a sound instance.
    fn stop(&mut self, id: SoundInstanceId) -> Result<(), EmeraldError> {
        if let Some(mut instance_handle) = self.instances.remove(&id) {
            instance_handle.stop(Default::default())?;
        }

        Ok(())
    }

    /// Pause a sound instance.
    fn pause(&mut self, id: SoundInstanceId) -> Result<(), EmeraldError> {
        if let Some(instance_handle) = self.instances.get_mut(&id) {
            instance_handle.pause(Default::default())?;
        }

        Ok(())
    }

    /// Resume a paused sound instance.
    fn resume(&mut self, id: SoundInstanceId) -> Result<(), EmeraldError> {
        if let Some(instance_handle) = self.instances.get_mut(&id) {
            instance_handle.resume(Default::default())?;
        }

        Ok(())
    }

    /// Clears all sounds and instances from the mixer.
    fn clear(&mut self) -> Result<(), EmeraldError> {
        for instance in self.instances.values_mut() {
            instance.stop(Default::default())?;
        }

        self.instances = HashMap::new();

        Ok(())
    }

    fn post_update(&mut self) -> Result<(), EmeraldError> {
        self.inner.free_unused_resources();

        let mut to_remove = Vec::new();

        for (id, instance) in &self.instances {
            if instance.state() == InstanceState::Stopped {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            self.instances.remove(&id);
        }

        Ok(())
    }
}
