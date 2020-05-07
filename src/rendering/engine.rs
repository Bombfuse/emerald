use miniquad::*;

pub struct RenderingEngine {
    
}
impl RenderingEngine {
    pub fn new() -> Self {
        RenderingEngine {

        }
    }

    pub fn update(&mut self, mut ctx: &mut Context) {
        let color = (100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0);
        ctx.clear(Some(color), None, None);
        ctx.commit_frame();
    }
}