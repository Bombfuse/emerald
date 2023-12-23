use crate::{asset_key::AssetKey, EmeraldError};

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub enum SoundFormat {
    Ogg,
    Wav,
}

/// An instance of a sound playing. Use this id to pause, play, and resume this sound instance.
#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct SoundInstanceId(usize);
impl SoundInstanceId {
    pub fn new(id: usize) -> Self {
        SoundInstanceId(id)
    }
}

/// A key to sound data in the engine.
#[derive(Clone, Debug)]
pub struct SoundKey {
    asset_key: AssetKey,
    format: SoundFormat,
}
impl SoundKey {
    pub fn new(asset_key: AssetKey, format: SoundFormat) -> Self {
        SoundKey { asset_key, format }
    }

    pub fn asset_key(&self) -> &AssetKey {
        &self.asset_key
    }
}

pub use sound_backend::*;

// Dummy sound backend
#[cfg(not(feature = "audio"))]
mod sound_backend {
    use crate::audio::sound::*;

    #[derive(Clone)]
    pub struct Sound {}
    impl Sound {
        pub(crate) fn new(_bytes: Vec<u8>, _format: SoundFormat) -> Result<Self, EmeraldError> {
            Ok(Sound {})
        }
    }
}
