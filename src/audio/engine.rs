use crate::{EmeraldError, audio::*};
use std::collections::HashMap;

pub(crate) struct AudioEngine {
    mixers: HashMap<String, Mixer>,
}
impl AudioEngine {
    pub(crate) fn new() -> Self {
        AudioEngine {
            mixers: HashMap::new(),
        }
    }
    
    pub(crate) fn mixer<T: Into<String>>(&mut self, mixer_name: T) -> Result<&mut Mixer, EmeraldError> {
        let mixer_name: String = mixer_name.into();

        if !self.mixers.contains_key(&mixer_name) {
            self.mixers.insert(mixer_name.clone(), Mixer::new()?);
        }

        if let Some(mixer) = self.mixers.get_mut(&mixer_name) {
            return Ok(mixer);
        }

        Err(EmeraldError::new(format!("Error creating and/or retrieving the mixer: {:?}", mixer_name)))
    }

    pub(crate) fn clear(&mut self) {
        for (_, mixer) in &mut self.mixers {
            mixer.clear();
        }

        self.mixers = HashMap::new();
    }
}
