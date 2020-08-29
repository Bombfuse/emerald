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

    pub fn play(&mut self, snd: Sound) -> SoundId {
        self.audio_engine.play(snd)
    }

    pub fn clear(&mut self) {
        self.audio_engine.clear();
    }

    /// This will play a sound and continuously loop it.
    pub fn play_and_loop(&mut self, snd: Sound) {

    }
}