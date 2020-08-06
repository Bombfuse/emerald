use crate::audio::*;

pub struct AudioHandler<'a> {
    audio_engine: &'a mut AudioEngine,
}
impl<'a> AudioHandler<'a> {
    pub(crate) fn new(audio_engine: &'a mut AudioEngine) -> Self {
        AudioHandler {
            audio_engine,
        }
    }

    pub fn play(&mut self, snd: Sound) {
        self.audio_engine.play(snd);
    }
}