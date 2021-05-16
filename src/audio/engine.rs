
use crate::{EmeraldError, audio::*};
use std::collections::HashMap;

pub(crate) struct AudioEngine {
    mixers: HashMap<String, Box<dyn Mixer>>,
}
impl AudioEngine {
    pub(crate) fn new() -> Self {
        AudioEngine {
            mixers: HashMap::new(),
        }
    }
    
    pub(crate) fn mixer<T: Into<String>>(&mut self, mixer_name: T) -> Result<&mut Box<dyn Mixer>, EmeraldError> {
        let mixer_name: String = mixer_name.into();

        if !self.mixers.contains_key(&mixer_name) {
            self.mixers.insert(mixer_name.clone(), crate::audio::mixer::new_mixer()?);
        }

        if let Some(mixer) = self.mixers.get_mut(&mixer_name) {
            return Ok(mixer);
        }

        Err(EmeraldError::new(format!("Error creating and/or retrieving the mixer: {:?}", mixer_name)))
    }

    pub(crate) fn post_update(&mut self) -> Result<(), EmeraldError> {
        for (_, mixer) in &mut self.mixers {
            mixer.post_update()?;
        }

        Ok(())
    }

    pub(crate) fn clear(&mut self) -> Result<(), EmeraldError> {
        for (_, mixer) in &mut self.mixers {
            mixer.clear()?;
        }

        self.mixers = HashMap::new();
        Ok(())
    }
}
