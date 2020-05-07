use crate::world::*;

use paintbrush::*;
use miniquad::*;

pub struct RenderingEngine {
    
}
impl RenderingEngine {
    pub fn new() -> Self {
        RenderingEngine {

        }
    }

    pub fn update(&mut self, ctx: &mut Context, world: &mut World) {
        ctx.clear(Some(CORNFLOWER_BLUE.percentage()), None, None);
        ctx.commit_frame();
    }

    fn draw_rect(&mut self, ctx: &mut Context) {}
}