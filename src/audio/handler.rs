use crate::EmeraldError;
use crate::{audio::*, AssetStore};

pub struct AudioHandler<'a> {
    audio_engine: &'a mut AudioEngine,
    asset_store: &'a mut AssetStore,
}
impl<'a> AudioHandler<'a> {
    pub(crate) fn new(audio_engine: &'a mut AudioEngine, asset_store: &'a mut AssetStore) -> Self {
        AudioHandler {
            audio_engine,
            asset_store,
        }
    }

    pub fn mixer<T: Into<String>>(&mut self, mixer_name: T) -> Result<MixerHandler, EmeraldError> {
        let mixer_name: String = mixer_name.into();

        if let Ok(mixer) = self.audio_engine.mixer(mixer_name.clone()) {
            return Ok(MixerHandler::new(mixer, &mut self.asset_store));
        }

        Err(EmeraldError::new(format!(
            "Unable to create and/or retrieve the mixer named: {}",
            mixer_name
        )))
    }

    /// Deletes and clears all mixers
    pub fn clear(&mut self) -> Result<(), EmeraldError> {
        self.audio_engine.clear()?;

        Ok(())
    }
}
