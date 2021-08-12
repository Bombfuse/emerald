use crate::{
    audio::{Mixer, SoundInstanceId, SoundKey},
    AssetStore, EmeraldError,
};

pub struct DummyMixer {}
impl DummyMixer {
    pub fn new() -> Result<Box<Self>, EmeraldError> {
        Ok(Box::new(DummyMixer {}))
    }
}

impl Mixer for DummyMixer {
    fn play(
        &mut self,
        _key: SoundKey,
        _asset_store: &mut AssetStore,
    ) -> Result<SoundInstanceId, EmeraldError> {
        Ok(SoundInstanceId::new(0))
    }

    fn play_and_loop(
        &mut self,
        _key: SoundKey,
        _asset_store: &mut AssetStore,
    ) -> Result<SoundInstanceId, EmeraldError> {
        Ok(SoundInstanceId::new(0))
    }
    fn get_volume(&self) -> Result<f32, EmeraldError> {
        Ok(0.0)
    }
    fn set_volume(&mut self, _volume: f32) -> Result<(), EmeraldError> {
        Ok(())
    }
    fn get_instances(&self) -> Result<Vec<SoundInstanceId>, EmeraldError> {
        Ok(Vec::new())
    }
    fn stop(&mut self, _snd_instance_id: SoundInstanceId) -> Result<(), EmeraldError> {
        Ok(())
    }
    fn pause(&mut self, _snd_instance_id: SoundInstanceId) -> Result<(), EmeraldError> {
        Ok(())
    }
    fn resume(&mut self, _snd_instance_id: SoundInstanceId) -> Result<(), EmeraldError> {
        Ok(())
    }
    fn clear(&mut self) -> Result<(), EmeraldError> {
        Ok(())
    }
    fn post_update(&mut self) -> Result<(), EmeraldError> {
        Ok(())
    }
}
