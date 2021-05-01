use std::collections::HashMap;

use kira::{instance::{self, InstanceId, InstanceSettings, handle::InstanceHandle}, manager::{AudioManager, error::SetupError}, sound::{Sound, SoundId, handle::SoundHandle}};

use crate::{EmeraldError};

pub struct Mixer {
    inner: AudioManager,
    sounds: HashMap<SoundId, Sound>,
    instances: HashMap<InstanceId, InstanceHandle>,
    volume: f64,
}
impl Mixer {
    pub fn new() -> Result<Self, EmeraldError> {
        Ok(Mixer {
            inner: AudioManager::new(Default::default())?,
            sounds: HashMap::new(),
            instances: HashMap::new(),
            volume: 1.0,
        })
    }

    /// Plays the given sound data once.
    /// Applies the volume of the mixer to the sound.
    pub fn play(&mut self, sound: Sound) -> Result<InstanceId, EmeraldError> {
        let mut sound_handle = self.inner.add_sound(sound)?;
        let instance_handle = sound_handle.play(InstanceSettings::new()
            .volume(self.volume)    
        )?;

        let id = instance_handle.id();
        self.instances.insert(id, instance_handle);

        Ok(id)
    }

    pub fn play_and_loop(&mut self, sound: Sound) -> Result<InstanceId, EmeraldError> {
        let mut sound_handle = self.inner.add_sound(sound)?;
        let instance_handle = sound_handle.play(InstanceSettings::new()
            .volume(self.volume)
            .loop_start(instance::InstanceLoopStart::Custom(0.0))
        )?;
        
        let id = instance_handle.id();
        self.instances.insert(id, instance_handle);

        Ok(id)
    }

    pub fn get_volume(&self) -> f64 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f64) -> Result<(), EmeraldError> {
        self.volume = volume;

        for (_, instance) in &mut self.instances {
            instance.set_volume(self.volume)?;
        }

        Ok(())
    }

    /// Get the ids of all instances in the mixer.
    pub fn instances(&self) -> Vec<InstanceId> {
        let mut instances = Vec::new();

        for key in self.instances.keys() {
            instances.push(*key);
        }

        instances
    }

    /// Stop a sound instance.
    pub fn stop(&mut self, id: InstanceId) -> Result<(), EmeraldError> {
        if let Some(mut instance_handle) = self.instances.remove(&id) {
            instance_handle.stop(Default::default())?;
        }

        Ok(())
    }

    /// Pause a sound instance.
    pub fn pause(&mut self, id: InstanceId) -> Result<(), EmeraldError> {
        if let Some(instance_handle) = self.instances.get_mut(&id) {
            instance_handle.pause(Default::default())?;
        }

        Ok(())
    }

    /// Resume a paused sound instance.
    pub fn resume(&mut self, id: InstanceId) -> Result<(), EmeraldError> {
        if let Some(instance_handle) = self.instances.get_mut(&id) {
            instance_handle.resume(Default::default())?;
        }

        Ok(())
    }

    /// Clears all sounds and instances from the mixer.
    pub fn clear(&mut self) -> Result<(), EmeraldError> {
        for (key, instance) in &mut self.instances {
            instance.stop(Default::default())?;
        }

        self.instances = HashMap::new();

        Ok(())
    }
}
