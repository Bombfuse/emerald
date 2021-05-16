use crate::EmeraldError;



#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub enum SoundFormat {
    Ogg,
    Wav,
}

/// An instance of a sound playing. Use this id to pause, play, and resume this sound instance.
#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct SoundInstanceId(usize);
impl SoundInstanceId {
    pub(crate) fn new(id: usize) -> Self {
        SoundInstanceId(id)
    }
}


/// A key to sound data in the engine.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct SoundKey(String, SoundFormat);
impl SoundKey {
    pub fn new<T: Into<String>>(sound_path: T, format: SoundFormat) -> Self {
        SoundKey(sound_path.into(), format)
    }
}

pub use sound_backend::*;


// Kira sound backend
#[cfg(feature = "audio")]
mod sound_backend {
    use crate::audio::sound::*;

    #[derive(Clone)]
    pub struct Sound {
        pub(crate) inner: kira::sound::Sound,
    }
    impl Sound {
        pub(crate) fn new(bytes: Vec<u8>, format: SoundFormat) -> Result<Self, EmeraldError> {
            let reader = std::io::Cursor::new(bytes);
            let settings = kira::sound::SoundSettings::new();

            let inner = match format {
                SoundFormat::Ogg => kira::sound::Sound::from_ogg_reader(reader, settings),
                SoundFormat::Wav => kira::sound::Sound::from_wav_reader(reader, settings),
            }?;

            Ok(Sound {
                inner,
            })
        }
    }
}

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