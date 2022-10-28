// // Credit to not-fl3/macroquad text: https://github.com/not-fl3/macroquad/blob/0.3/src/text.rs
// use crate::rendering::*;
// use crate::texture::TextureKey;
// use crate::{AssetStore, Color, EmeraldError};

// use fontdue::layout::GlyphRasterConfig;
// pub use fontdue::layout::{HorizontalAlign, VerticalAlign, WrapStyle};

// use std::collections::HashMap;

use std::collections::HashMap;

use fontdue::layout::GlyphRasterConfig;

use crate::{
    rendering_engine::RenderingEngine, texture::TextureKey, AssetStore, Color, EmeraldError,
};

// TODO: We should include a real default font with the game engine
pub(crate) const DEFAULT_FONT_TEXTURE_PATH: &str = "ghosty_spooky_mister_mime_dude";

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FontKey(pub(crate) String, pub(crate) u32);
impl FontKey {
    pub fn new<T: Into<String>>(font_path: T, font_size: u32) -> Self {
        FontKey(font_path.into(), font_size)
    }
}
impl Default for FontKey {
    fn default() -> FontKey {
        FontKey::new(DEFAULT_FONT_TEXTURE_PATH, 40)
    }
}

pub(crate) struct CharacterInfo {
    pub offset_x: i32,
    pub _offset_y: i32,
    pub _advance: f32,

    pub glyph_x: u32,
    pub glyph_y: u32,
    pub glyph_w: u32,
    pub glyph_h: u32,
}

pub(crate) struct FontImage {
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

pub(crate) struct Font {
    pub _font_key: FontKey,
    pub characters: HashMap<GlyphRasterConfig, CharacterInfo>,
    pub font_texture_key: TextureKey,
    pub font_image: FontImage,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub max_line_height: u16,
}
impl Font {
    pub const GAP: u16 = 2;

    pub fn new(
        font_key: FontKey,
        font_texture_key: TextureKey,
        font_image: FontImage,
    ) -> Result<Font, EmeraldError> {
        Ok(Font {
            font_image,
            _font_key: font_key,
            font_texture_key,
            characters: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            max_line_height: 0,
        })
    }
}

pub(crate) fn cache_glyph(
    rendering_engine: &mut RenderingEngine,
    asset_store: &mut AssetStore,
    font_key: &FontKey,
    glyph_key: GlyphRasterConfig,
    size: u16,
) -> Result<(), EmeraldError> {
    let mut recache_characters = None;
    let mut update_font_texture = false;

    let mut optional_metrics = None;
    let mut optional_bitmap = None;

    let mut to_update = Vec::new();

    if let Some(font) = asset_store.get_fontdue_font(font_key) {
        let (metrics, bitmap) = font.rasterize_config(glyph_key);
        optional_metrics = Some(metrics);
        optional_bitmap = Some(bitmap);
    }

    if let (Some(metrics), Some(bitmap)) = (optional_metrics, optional_bitmap) {
        if let Some(font) = asset_store.get_font_mut(font_key) {
            if metrics.advance_height != 0.0 {
                return Err(EmeraldError::new("Vertical fonts are not supported"));
            }

            let (width, height) = (metrics.width, metrics.height);
            let advance = metrics.advance_width;
            let (offset_x, offset_y) = (metrics.xmin, metrics.ymin);

            let x = if font.cursor_x + (width as u16) < font.font_image.width {
                if height as u16 > font.max_line_height {
                    font.max_line_height = height as u16;
                }
                let res = font.cursor_x;
                font.cursor_x += width as u16 + Font::GAP;
                res
            } else {
                font.cursor_y += font.max_line_height + Font::GAP;
                font.cursor_x = width as u16 + Font::GAP;
                font.max_line_height = height as u16;
                Font::GAP
            };

            let y = font.cursor_y;

            let character_info = CharacterInfo {
                glyph_x: x as _,
                glyph_y: y as _,
                glyph_w: width as _,
                glyph_h: height as _,

                _advance: advance,
                offset_x,
                _offset_y: offset_y,
            };

            font.characters.insert(glyph_key, character_info);

            // texture bounds exceeded
            if font.cursor_y + height as u16 > font.font_image.height {
                // reset glyph asset_store state
                let characters = font.characters.drain().collect::<Vec<_>>();
                font.cursor_x = 0;
                font.cursor_y = 0;
                font.max_line_height = 0;

                // increase font texture size
                font.font_image = FontImage::gen_image_color(
                    font.font_image.width * 2,
                    font.font_image.height * 2,
                    Color::new(0, 0, 0, 0),
                );

                to_update.push((font.font_image.bytes.clone(), font.font_texture_key.clone()));
                recache_characters = Some(characters);
            } else {
                for j in 0..height {
                    for i in 0..width {
                        let coverage = bitmap[j * width + i];
                        font.font_image.set_pixel(
                            x as u32 + i as u32,
                            y as u32 + j as u32,
                            Color::new(255, 255, 255, coverage),
                        );
                    }
                }

                update_font_texture = true;
            }
        }
    }

    for (bytes, key) in to_update {
        rendering_engine.load_texture(asset_store, &bytes, key)?;
    }

    if update_font_texture {
        rendering_engine.update_font_texture(asset_store, font_key);
    }

    if let Some(characters) = recache_characters {
        // recache all previously asset_stored symbols
        for (glyph_key, _) in characters {
            cache_glyph(rendering_engine, asset_store, font_key, glyph_key, size)?;
        }
    }

    Ok(())
}
