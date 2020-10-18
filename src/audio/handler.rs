use crate::audio::*;
use crate::{EmeraldError};

pub struct AudioHandler<'a> {
    audio_engine: &'a mut AudioEngine,
}
impl<'a> AudioHandler<'a> {
    pub(crate) fn new(audio_engine: &'a mut AudioEngine) -> Self {
        AudioHandler {
            audio_engine,
        }
    }

    pub fn mixer<T: Into<String>>(&mut self, mixer_name: T) -> Result<&mut Mixer, EmeraldError> {
        let mixer_name: String = mixer_name.into();

        if let Some(mixer) = self.audio_engine.mixer(mixer_name.clone()) {
            return Ok(mixer);
        }
        
        Err(EmeraldError::new(format!("Unable to create and/or retrieve the mixer named: {}", mixer_name)))
    }

    /// Deletes and clears all mixers
    pub fn clear(&mut self) {
        self.audio_engine.clear();
    }
}