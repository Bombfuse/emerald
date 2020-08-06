
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct SoundKey(String, SoundFormat);
impl SoundKey {
    pub fn new<T: Into<String>>(texture_path: T, format: SoundFormat) -> Self {
        SoundKey(texture_path.into(), format)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum SoundFormat {
    Ogg,
    Wav
}