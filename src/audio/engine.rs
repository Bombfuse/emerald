use quad_snd::{
    decoder::{read_ogg, read_wav},
    mixer::{SoundMixer, Sound},
};

use crate::{EmeraldError};
use crate::audio::*;

use std::fs::File;
use std::collections::HashMap;
use std::io::Read as ReadFile;

pub(crate) struct AudioEngine {
    mixer: SoundMixer,
}
impl AudioEngine {
    pub(crate) fn new() -> Self {
        let mixer = SoundMixer::new();

        AudioEngine {
            mixer,
        }
    }

    pub(crate) fn load(&mut self, mut sound_file: File, sound_format: SoundFormat) -> Result<Sound, EmeraldError> {
        let mut sound_bytes = Vec::new();
        sound_file.read_to_end(&mut sound_bytes)?;

        let sound = match sound_format {
            SoundFormat::Ogg => read_ogg(sound_bytes.as_slice()).unwrap(),
            SoundFormat::Wav => read_wav(sound_bytes.as_slice()).unwrap(),
        };

        Ok(sound)
    }

    pub(crate) fn play(&mut self, mut snd: Sound) {
        self.mixer.play(snd);
    }

    pub(crate) fn frame(&mut self) {
        self.mixer.frame();
    }
}