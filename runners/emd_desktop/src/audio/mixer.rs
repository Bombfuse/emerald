use emerald::{EmeraldError, Mixer, SoundInstanceId, SoundKey, ThreadSafeMixer};

use kira::{
    manager::{backend::DefaultBackend, AudioManager},
    sound::{static_sound::StaticSoundData, Sound, SoundData},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

static mut KIRA_AUDIO_MANAGER: Option<Arc<Mutex<AudioManager>>> = None;

pub(crate) struct KiraMixer {
    inner: Arc<Mutex<AudioManager>>,
    volume: f64,
    sound_instance_uid: usize,
}
unsafe impl Send for KiraMixer {}
unsafe impl Sync for KiraMixer {}
impl KiraMixer {
    pub fn new(inner: Arc<Mutex<AudioManager>>) -> Result<ThreadSafeMixer, EmeraldError> {
        Ok(Box::new(KiraMixer {
            inner,
            volume: 1.0,
            sound_instance_uid: 0,
        }))
    }

    fn get_inner_handle(&mut self) -> Result<MutexGuard<'_, AudioManager>, EmeraldError> {
        match self.inner.lock() {
            Ok(inner) => Ok(inner),
            Err(e) => Err(EmeraldError::new(format!(
                "Error while trying to retrieve a handle on the audio manager. {:?}",
                e
            ))),
        }
    }
}
impl Mixer for KiraMixer {
    fn play(
        &mut self,
        sound: &SoundKey,
        asset_store: &mut emerald::AssetEngine,
    ) -> Result<emerald::SoundInstanceId, EmeraldError> {
        unsafe {
            KIRA_AUDIO_MANAGER.as_mut().map(|a| {
                a.lock().ok().map(|mut manager| {
                    asset_store
                        .get_asset::<StaticSoundData>(&sound.asset_key().asset_id())
                        .map(|data| {
                            manager.play(data.clone()).ok();
                        });
                });
            });
        }
        self.sound_instance_uid += 1;
        let sound_instance_id = self.sound_instance_uid;
        Ok(SoundInstanceId::new(sound_instance_id))
    }

    fn play_and_loop(
        &mut self,
        sound: &SoundKey,
        asset_store: &mut emerald::AssetEngine,
    ) -> Result<emerald::SoundInstanceId, EmeraldError> {
        self.play(sound, asset_store)
    }

    fn get_volume(&self) -> Result<f32, EmeraldError> {
        Ok(self.volume as f32)
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), EmeraldError> {
        self.volume = volume as f64;
        // TODO: set volume of all instances
        Ok(())
    }

    fn set_instance_volume(
        &mut self,
        snd_instance_id: emerald::SoundInstanceId,
        volume: f32,
    ) -> Result<(), EmeraldError> {
        todo!()
    }

    fn get_instances(&self) -> Result<Vec<emerald::SoundInstanceId>, EmeraldError> {
        todo!()
    }

    fn stop(&mut self, snd_instance_id: emerald::SoundInstanceId) -> Result<(), EmeraldError> {
        todo!()
    }

    fn pause(&mut self, snd_instance_id: emerald::SoundInstanceId) -> Result<(), EmeraldError> {
        todo!()
    }

    fn resume(&mut self, snd_instance_id: emerald::SoundInstanceId) -> Result<(), EmeraldError> {
        todo!()
    }

    fn clear(&mut self) -> Result<(), EmeraldError> {
        todo!()
    }

    fn post_update(&mut self) -> Result<(), EmeraldError> {
        Ok(())
    }
}

pub(crate) fn new_mixer() -> Result<ThreadSafeMixer, EmeraldError> {
    use kira::manager::AudioManagerSettings;

    unsafe {
        if KIRA_AUDIO_MANAGER.is_none() {
            let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings {
                ..Default::default()
            })
            .unwrap();
            KIRA_AUDIO_MANAGER = Some(Arc::new(Mutex::new(audio_manager)));
        }

        if let Some(audio_manager) = &mut KIRA_AUDIO_MANAGER {
            let mixer = KiraMixer::new(audio_manager.clone())?;

            return Ok(mixer);
        }
    }

    Err(EmeraldError::new(
        "Unable to find or create the kira audio manager",
    ))
}
