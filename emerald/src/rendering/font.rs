use std::collections::HashMap;

use fontdue::layout::GlyphRasterConfig;

use crate::{
    asset_key::AssetKey, rendering_engine::RenderingEngine, AssetEngine, Color, EmeraldError,
};

#[derive(Clone, Debug)]
pub struct FontKey {
    size: u32,
    path: String,
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

pub fn cache_glyph(
    rendering_engine: &mut Box<dyn RenderingEngine>,
    asset_engine: &mut AssetEngine,
    font_key: &FontKey,
    glyph_key: GlyphRasterConfig,
    size: u16,
) -> Result<(), EmeraldError> {
    // let mut recache_characters = None;
    let mut update_font_texture = false;

    let mut optional_metrics = None;
    let mut optional_bitmap = None;
    let mut recache_characters = None;

    let mut to_update = Vec::new();

    if let Some(font) = asset_engine.get_asset::<Font>(&font_key.asset_key.asset_id) {
        let (metrics, bitmap) = font.inner.rasterize_config(glyph_key);
        optional_metrics = Some(metrics);
        optional_bitmap = Some(bitmap);
    } else {
        return Err(EmeraldError::new(format!(
            "Unable to get Fontdue Font while caching font glyph: {:?}",
            font_key
        )));
    }

    if let (Some(metrics), Some(bitmap)) = (optional_metrics, optional_bitmap) {
        if let Some(font) = asset_engine.get_asset_mut::<Font>(&font_key.asset_key.asset_id) {
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

                to_update.push((font.font_image.bytes.clone(), font_key.path.clone()));
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
        } else {
            return Err(EmeraldError::new(format!(
                "Unable to get Font while caching font glyph: {:?}",
                font_key
            )));
        }
    } else {
        return Err(EmeraldError::new(format!(
            "Unable to get Metrics while caching font glyph: {:?}",
            font_key
        )));
    }

    for (bytes, label) in to_update {
        rendering_engine.load_texture(&label, asset_engine, &bytes)?;
    }

    if update_font_texture {
        rendering_engine.update_font_texture(asset_engine, font_key)?;
    }

    if let Some(characters) = recache_characters {
        // recache all previously asset_stored symbols
        for (glyph_key, _) in characters {
            cache_glyph(rendering_engine, asset_engine, font_key, glyph_key, size)?;
        }
    }

    Ok(())
}
