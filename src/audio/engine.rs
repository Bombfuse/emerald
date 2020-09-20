use quad_snd::{
    decoder::{read_ogg, read_wav},
    mixer::{SoundMixer, Sound},
};

use crate::{EmeraldError};
use crate::audio::*;

pub(crate) struct AudioEngine {
    mixer: SoundMixer,
    sound_ids: Vec<SoundId>,
}
impl AudioEngine {
    pub(crate) fn new() -> Self {
        let mixer = SoundMixer::new();

        AudioEngine {
            mixer,
            sound_ids: Vec::new(),
        }
    }

    pub(crate) fn load(&mut self, sound_bytes: Vec<u8>, sound_format: SoundFormat) -> Result<Sound, EmeraldError> {
        let sound = match sound_format {
            SoundFormat::Ogg => read_ogg(sound_bytes.as_slice()).unwrap(),
            SoundFormat::Wav => read_wav(sound_bytes.as_slice()).unwrap(),
        };

        Ok(sound)
    }

    pub(crate) fn play(&mut self, snd: Sound) -> SoundId {
        let id = self.mixer.play(snd);

        self.sound_ids.push(id);

        id
    }

    pub(crate) fn clear(&mut self) {
        let ids = self.sound_ids.clone();
        self.sound_ids = Vec::new();


        for id in ids {
            self.stop(id);
        }
    }

    pub(crate) fn stop(&mut self, id: SoundId) {
        self.mixer.stop(id)
    }

    pub(crate) fn frame(&mut self) {
        self.mixer.frame();
    }
}