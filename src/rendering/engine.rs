use crate::world::*;
use crate::rendering::*;

use paintbrush::*;
use miniquad::*;

pub struct RenderingEngine {
    settings: RenderSettings,
}
impl RenderingEngine {
    pub fn new(settings: RenderSettings) -> Self {
        RenderingEngine {
            settings,
        }
    }

    pub fn update(&mut self, ctx: &mut Context, world: &mut World) {
        ctx.clear(Some(self.settings.background_color.percentage()), None, None);
        ctx.commit_frame();
    }

    fn draw_rect(&mut self, ctx: &mut Context) {}
}