use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct RenderSettings {
    pub background_color: Color,
}
impl Default for RenderSettings {
    fn default() -> RenderSettings {
        RenderSettings {
            background_color: CORNFLOWER_BLUE,
        }
    }
}