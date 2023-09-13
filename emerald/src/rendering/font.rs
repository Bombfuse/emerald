use std::collections::HashMap;

use fontdue::layout::GlyphRasterConfig;

use crate::{
    asset_key::AssetKey, rendering_engine::RenderingEngine, AssetEngine, Color, EmeraldError,
};

#[derive(Clone, Debug)]
pub struct FontKey {
    size: u32,
    pub(crate) path: String,
    pub(crate) asset_key: AssetKey,
}
impl FontKey {
    pub fn new<T: Into<String>>(asset_key: AssetKey, font_path: T, font_size: u32) -> Self {
        FontKey {
            asset_key,
            path: font_path.into(),
            size: font_size,
        }
    }

    pub fn asset_key(&self) -> &AssetKey {
        &self.asset_key
    }
}

pub struct CharacterInfo {
    pub offset_x: i32,
    pub _offset_y: i32,
    pub _advance: f32,

    pub glyph_x: u32,
    pub glyph_y: u32,
    pub glyph_w: u32,
    pub glyph_h: u32,
}

pub struct FontImage {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}
impl FontImage {
    pub fn gen_image_color(width: u16, height: u16, color: Color) -> Self {
        let mut bytes = vec![0; width as usize * height as usize * 4];

        for i in 0..width as usize * height as usize {
            bytes[i * 4] = color.r;
            bytes[i * 4 + 1] = color.g;
            bytes[i * 4 + 2] = color.b;
            bytes[i * 4 + 3] = color.a;
        }

        FontImage {
            width,
            height,
            bytes,
        }
    }

    pub fn get_image_data_mut(&mut self) -> &mut [Color] {
        use std::slice;

        unsafe {
            slice::from_raw_parts_mut(
                self.bytes.as_mut_ptr() as *mut Color,
                self.width as usize * self.height as usize,
            )
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let width = self.width;

        self.get_image_data_mut()[(y * width as u32 + x) as usize] = color;
    }
}

pub struct Font {
    pub characters: HashMap<GlyphRasterConfig, CharacterInfo>,
    pub font_texture_key: AssetKey,
    pub font_image: FontImage,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub max_line_height: u16,
    pub inner: fontdue::Font,
}
impl Font {
    pub const GAP: u16 = 2;

    pub fn new(
        inner: fontdue::Font,
        font_texture_key: AssetKey,
        font_image: FontImage,
    ) -> Result<Font, EmeraldError> {
        Ok(Font {
            font_image,
            inner,
            font_texture_key,
            characters: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            max_line_height: 0,
        })
    }
}
