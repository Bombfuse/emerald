use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct RenderSettings {
    pub background_color: Color,
    pub fullscreen: bool,
    pub window_size: (u32, u32),
}
impl Default for RenderSettings {
    fn default() -> RenderSettings {
        RenderSettings {
            background_color: CORNFLOWER_BLUE,
            fullscreen: false,
            window_size: (800, 600),
        }
    }
}
