use quad_snd::{
    decoder::{read_ogg, read_wav},
    mixer::Sound,
};

use crate::audio::*;
use crate::EmeraldError;

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

    pub(crate) fn load(
        &mut self,
        sound_bytes: Vec<u8>,
        sound_format: SoundFormat,
    ) -> Result<Sound, EmeraldError> {
        let sound = match sound_format {
            SoundFormat::Ogg => read_ogg(sound_bytes.as_slice()).unwrap(),
            SoundFormat::Wav => read_wav(sound_bytes.as_slice()).unwrap(),
        };

        Ok(sound)
    }

    pub(crate) fn mixer<T: Into<String>>(&mut self, mixer_name: T) -> Option<&mut Mixer> {
        let mixer_name: String = mixer_name.into();

        if !self.mixers.contains_key(&mixer_name) {
            self.mixers.insert(mixer_name.clone(), Mixer::new());
        }

        self.mixers.get_mut(&mixer_name)
    }

    pub(crate) fn clear(&mut self) {
        for (_, mixer) in &mut self.mixers {
            mixer.clear();
        }

        self.mixers = HashMap::new();
    }

    pub(crate) fn frame(&mut self) {
        for (_, mixer) in &mut self.mixers {
            mixer.frame();
        }
    }
}
