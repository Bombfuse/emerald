use crate::{audio::*, AssetEngine, EmeraldError};

pub trait AudioEngine {
    fn initialize(&mut self, asset_engine: &mut AssetEngine);

    /// Fetches or creates a mixer with the given name.
    fn mixer(&mut self, mixer_name: &str) -> Result<&mut ThreadSafeMixer, EmeraldError>;

    /// Called at the end of each frame.
    fn post_update(&mut self) -> Result<(), EmeraldError>;

    /// Clear all mixers and audio.
    fn clear(&mut self) -> Result<(), EmeraldError>;
}
