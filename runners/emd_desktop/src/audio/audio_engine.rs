use emerald::{assets::asset_engine::AssetEngine, AudioEngine};
pub struct DesktopAudioEngine {}
impl AudioEngine for DesktopAudioEngine {
    fn initialize(&mut self, asset_engine: &mut AssetEngine) {}

    fn mixer(
        &mut self,
        mixer_name: &str,
    ) -> Result<&mut emerald::ThreadSafeMixer, emerald::EmeraldError> {
        todo!()
    }

    fn post_update(&mut self) -> Result<(), emerald::EmeraldError> {
        Ok(())
    }

    fn clear(&mut self) -> Result<(), emerald::EmeraldError> {
        Ok(())
    }
}
