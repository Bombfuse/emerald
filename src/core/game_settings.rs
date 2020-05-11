use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct GameSettings {
    pub render_settings: RenderSettings,
}
impl Default for GameSettings {
    fn default() -> GameSettings {
        GameSettings {
            render_settings: RenderSettings::default(),
        }
    }
}