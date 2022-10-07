use crate::{audio::sound::SoundInstanceId, AssetStore, EmeraldError, SoundKey};

#[cfg(feature = "audio")]
mod kira_backend;

#[cfg(feature = "audio")]
use kira::manager::AudioManager;
#[cfg(feature = "audio")]
use kira_backend::KiraMixer as BackendMixer;

#[cfg(not(feature = "audio"))]
mod dummy;
#[cfg(not(feature = "audio"))]
use dummy::DummyMixer as BackendMixer;

#[cfg(target_arch = "wasm32")]
pub(crate) type ThreadSafeMixer = Box<dyn Mixer>;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type ThreadSafeMixer = Box<dyn Mixer + Send + Sync>;

pub(crate) trait Mixer {
    fn play(
        &mut self,
        sound: SoundKey,
        asset_store: &mut AssetStore,
    ) -> Result<SoundInstanceId, EmeraldError>;
    fn play_and_loop(
        &mut self,
        sound: SoundKey,
        asset_store: &mut AssetStore,
    ) -> Result<SoundInstanceId, EmeraldError>;
    fn get_volume(&self) -> Result<f32, EmeraldError>;
    fn set_volume(&mut self, volume: f32) -> Result<(), EmeraldError>;
    fn set_instance_volume(
        &mut self,
        snd_instance_id: SoundInstanceId,
        volume: f32,
    ) -> Result<(), EmeraldError>;
    fn get_instances(&self) -> Result<Vec<SoundInstanceId>, EmeraldError>;
    fn stop(&mut self, snd_instance_id: SoundInstanceId) -> Result<(), EmeraldError>;
    fn pause(&mut self, snd_instance_id: SoundInstanceId) -> Result<(), EmeraldError>;
    fn resume(&mut self, snd_instance_id: SoundInstanceId) -> Result<(), EmeraldError>;
    fn clear(&mut self) -> Result<(), EmeraldError>;
    fn post_update(&mut self) -> Result<(), EmeraldError>;
}

#[cfg(not(feature = "audio"))]
pub(crate) fn new_mixer() -> Result<ThreadSafeMixer, EmeraldError> {
    let mixer = BackendMixer::new()?;

    Ok(mixer)
}

#[cfg(feature = "audio")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "audio")]
static mut KIRA_AUDIO_MANAGER: Option<Arc<Mutex<AudioManager>>> = None;

#[cfg(feature = "audio")]
pub(crate) fn new_mixer() -> Result<ThreadSafeMixer, EmeraldError> {
    use kira::manager::AudioManagerSettings;

    unsafe {
        if KIRA_AUDIO_MANAGER.is_none() {
            let audio_manager = AudioManager::new(AudioManagerSettings {
                num_sounds: 1000,
                num_instances: 1000,
                ..Default::default()
            })?;
            KIRA_AUDIO_MANAGER = Some(Arc::new(Mutex::new(audio_manager)));
        }

        if let Some(audio_manager) = &mut KIRA_AUDIO_MANAGER {
            let mixer = BackendMixer::new(audio_manager.clone())?;

            return Ok(mixer);
        }
    }

    Err(EmeraldError::new(
        "Unable to find or creat the kira audio manager",
    ))
}

pub struct MixerHandler<'a> {
    inner: &'a mut ThreadSafeMixer,
    asset_store: &'a mut AssetStore,
}
impl<'a> MixerHandler<'a> {
    pub(crate) fn new(inner: &'a mut ThreadSafeMixer, asset_store: &'a mut AssetStore) -> Self {
        MixerHandler { inner, asset_store }
    }

    pub fn play(&mut self, key: SoundKey) -> Result<SoundInstanceId, EmeraldError> {
        self.inner.play(key, &mut self.asset_store)
    }
    pub fn play_and_loop(&mut self, key: SoundKey) -> Result<SoundInstanceId, EmeraldError> {
        self.inner.play_and_loop(key, &mut self.asset_store)
    }
    pub fn get_volume(&self) -> Result<f32, EmeraldError> {
        self.inner.get_volume()
    }
    pub fn set_instance_volume(
        &mut self,
        snd_instance_id: SoundInstanceId,
        volume: f32,
    ) -> Result<(), EmeraldError> {
        self.inner.set_instance_volume(snd_instance_id, volume)
    }
    pub fn set_volume(&mut self, volume: f32) -> Result<(), EmeraldError> {
        self.inner.set_volume(volume)
    }
    pub fn get_instances(&self) -> Result<Vec<SoundInstanceId>, EmeraldError> {
        self.inner.get_instances()
    }
    pub fn stop(&mut self, snd_instance_id: SoundInstanceId) -> Result<(), EmeraldError> {
        self.inner.stop(snd_instance_id)
    }
    pub fn pause(&mut self, snd_instance_id: SoundInstanceId) -> Result<(), EmeraldError> {
        self.inner.pause(snd_instance_id)
    }
    pub fn resume(&mut self, snd_instance_id: SoundInstanceId) -> Result<(), EmeraldError> {
        self.inner.resume(snd_instance_id)
    }
    pub fn clear(&mut self) -> Result<(), EmeraldError> {
        self.inner.clear()
    }
    pub fn clear_sounds(&mut self) -> Result<(), EmeraldError> {
        for instance_id in self.get_instances()? {
            self.stop(instance_id)?;
        }
        Ok(())
    }
}
