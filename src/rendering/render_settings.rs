use crate::*;

#[derive(Clone, Copy, Debug)]
pub enum ScreenScalar {
    /// Perform no automatic scaling
    None,

    /// Keep the initial aspect ratio, providing black borders
    Keep,

    /// Stretch to fill screen
    Stretch,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderSettings {
    pub background_color: Color,
    pub fullscreen: bool,
    pub resolution: (u32, u32),
    pub high_dpi: bool,
    pub resizable_window: bool,
    pub icon: Option<miniquad::conf::Icon>,
}
impl Default for RenderSettings {
    fn default() -> RenderSettings {
        RenderSettings {
            background_color: CORNFLOWER_BLUE,
            fullscreen: false,
            resolution: (800, 600),
            high_dpi: false,
            resizable_window: true,
            icon: None,
        }
    }
}
