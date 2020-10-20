use quad_snd::mixer::{PlaybackStyle, Sound, SoundMixer, Volume};

use crate::audio::*;

pub struct Mixer {
    inner: SoundMixer,
    sound_ids: Vec<SoundId>,
    volume: f32,
}
impl Mixer {
    pub fn new() -> Self {
        Mixer {
            inner: SoundMixer::new(),
            sound_ids: Vec::new(),
            volume: 1.0,
        }
    }

    pub fn play(&mut self, snd: Sound) -> SoundId {
        let id = self.inner.play(snd);
        self.sound_ids.push(id);

        id
    }

    pub fn play_and_loop(&mut self, mut snd: Sound) -> SoundId {
        snd.playback_style = PlaybackStyle::Looped;
        let id = self.inner.play(snd);
        self.sound_ids.push(id);

        id
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.inner.set_volume_self(Volume(volume))
    }

    pub fn stop(&mut self, id: SoundId) {
        self.inner.stop(id)
    }

    pub fn clear(&mut self) {
        let ids = self.sound_ids.clone();
        for id in ids {
            self.stop(id);
        }

        self.sound_ids = Vec::new();
    }

    pub(crate) fn frame(&mut self) {
        self.inner.frame()
    }
}
