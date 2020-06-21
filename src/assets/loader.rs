use crate::*;
use crate::rendering::*;

pub struct AssetLoader<'a> {
    quad_ctx: &'a mut miniquad::Context,
    rendering_engine: &'a mut RenderingEngine,
}
impl<'a> AssetLoader<'a> {
    pub fn new(quad_ctx: &'a mut miniquad::Context, rendering_engine: &'a mut RenderingEngine) -> Self {
        AssetLoader {
            rendering_engine,
            quad_ctx,
        }
    }

    pub fn sprite(&mut self, path: &str) -> Result<Sprite, EmeraldError> {
        self.rendering_engine.sprite(&mut self.quad_ctx, path)
    }

    pub fn label(&mut self, path: &str, font_size: u16) -> Result<Label, EmeraldError> {
        let font_key = self.rendering_engine.font(&mut self.quad_ctx, path, font_size)?;
        let mut label = Label::default();
        label.font = font_key;

        Ok(Label::default())
    }
}