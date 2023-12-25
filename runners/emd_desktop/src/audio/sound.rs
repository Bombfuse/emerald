use emerald::SoundFormat;
use kira::sound::static_sound::StaticSoundData;

pub struct Sound {
    inner: StaticSoundData,
    format: SoundFormat,
}
