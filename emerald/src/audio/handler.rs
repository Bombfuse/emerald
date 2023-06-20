use crate::EmeraldError;
use crate::{audio::*, AssetEngine};

pub struct AudioHandler<'a> {
    audio_engine: &'a mut AudioEngine,
    asset_store: &'a mut AssetEngine,
}
impl<'a> AudioHandler<'a> {
    pub(crate) fn new(audio_engine: &'a mut AudioEngine, asset_store: &'a mut AssetEngine) -> Self {
        AudioHandler {
            audio_engine,
            asset_store,
        }
    }

    pub fn mixer(&mut self, mixer_name: &str) -> Result<MixerHandler<'_>, EmeraldError> {
        if let Ok(mixer) = self.audio_engine.mixer(mixer_name) {
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
