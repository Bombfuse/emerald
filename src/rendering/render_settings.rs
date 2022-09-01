use crate::*;

#[derive(Clone, Debug)]
pub struct RenderSettings {
    pub background_color: Color,
    pub fullscreen: bool,
    pub resolution: (u32, u32),
    pub high_dpi: bool,
    pub resizable_window: bool,
    pub icon: Option<Icon>,

    // Whether or not the game engine should automatically cull sprites that are not in camera view
    pub frustrum_culling: bool,
    pub pixel_snap: bool,
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
            frustrum_culling: true,
            pixel_snap: true,
        }
    }
}
