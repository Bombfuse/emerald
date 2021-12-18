use crate::*;

#[derive(Debug)]
pub struct GameSettings {
    pub title: String,
    pub render_settings: RenderSettings,
}
impl Default for GameSettings {
    fn default() -> GameSettings {
        GameSettings {
            title: String::from("Emerald"),
            render_settings: RenderSettings::default(),
        }
    }
}
