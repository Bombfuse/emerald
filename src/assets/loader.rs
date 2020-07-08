use crate::*;
use crate::rendering::*;

use std::fs::File;

pub struct AssetLoader<'a> {
    pub(crate) quad_ctx: &'a mut miniquad::Context,
    rendering_engine: &'a mut RenderingEngine,
}
impl<'a> AssetLoader<'a> {
    pub fn new(quad_ctx: &'a mut miniquad::Context, rendering_engine: &'a mut RenderingEngine) -> Self {
        AssetLoader {
            rendering_engine,
            quad_ctx,
        }
    }

    pub fn file<T: Into<String>>(&self, file_path: T) -> Result<File, EmeraldError> {
        let file_path: String = file_path.into();
        let file = File::open(file_path)?;

        Ok(file)
    }

    pub fn aseprite<T: Into<String>>(&mut self, path_to_texture: T, path_to_animations: T) -> Result<Aseprite, EmeraldError> {
        let texture_path = path_to_texture.into();
        let animation_path = path_to_animations.into();

        let texture_file = self.file(texture_path.clone())?;
        let animation_file = self.file(animation_path.clone())?;

        self.rendering_engine.aseprite(&mut self.quad_ctx,
            texture_file,
            texture_path,
            animation_file,
            animation_path)
    }

    pub fn sprite(&mut self, path: &str) -> Result<Sprite, EmeraldError> {
        let sprite_file = self.file(path)?;
        self.rendering_engine.sprite(&mut self.quad_ctx, sprite_file, path)
    }

    /// Meant to be used for WASM. Packs the textures into the WASM so
    /// that they can be loaded immediately without breaking the API.
    /// 
    /// emd.loader()
    ///     .set_texture(
    ///         "./assets/bunny.png",
    ///         include_bytes!("../static/assets/bunny.png").to_vec()
    ///     );
    /// 
    pub fn pack_texture(&mut self, name: &str, bytes: Vec<u8>) {
        self.rendering_engine.pack_texture(&mut self.quad_ctx, name, bytes)
    }

    pub fn label<T: Into<String>>(&mut self, text: T, font_key: FontKey) -> Result<Label, EmeraldError> {
        let mut label = Label::default();

        label.font = font_key;
        label.text = text.into();

        Ok(Label::default())
    }

    pub fn font(&mut self, path: &str, font_size: u16) -> Result<FontKey, EmeraldError> {
        let file = self.file(path)?;

        self.rendering_engine.font(&mut self.quad_ctx, file, path, font_size)
    }
}