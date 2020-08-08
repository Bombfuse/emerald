use crate::*;

#[derive(Clone, Copy, Debug)]
pub enum ScreenScalar {
    None,
    Keep,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderSettings {
    pub background_color: Color,
    pub fullscreen: bool,
    pub resolution: (u32, u32),
    pub scalar: ScreenScalar,
}
impl Default for RenderSettings {
    fn default() -> RenderSettings {
        RenderSettings {
            background_color: CORNFLOWER_BLUE,
            fullscreen: false,
            resolution: (800, 600),
            scalar: ScreenScalar::Keep,
        }
    }
}
