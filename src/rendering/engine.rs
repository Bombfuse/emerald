use crate::world::*;
use crate::rendering::*;

use paintbrush::*;
use miniquad::*;

pub struct RenderingEngine {
    settings: RenderSettings,
    pipeline: Pipeline,
    // bindings: Bindings,
}
impl RenderingEngine {
    pub fn new(ctx: &mut Context, settings: RenderSettings) -> Self {
        let shader = Shader::new(ctx, VERTEX, FRAGMENT, META);

        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
        );

        RenderingEngine {
            settings,
            pipeline,
            // bindings,
        }
    }

    pub fn update(&mut self, ctx: &mut Context, world: &mut World) {
        ctx.clear(Some(self.settings.background_color.percentage()), None, None);

        ctx.begin_default_pass(Default::default());
        ctx.apply_pipeline(&self.pipeline);

        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn render_color_rect(&mut self, ctx: &mut Context, color_rect: &ColorRect) {}

    fn render_sprite(&mut self, ctx: &mut Context, sprite: &Sprite) {}
}