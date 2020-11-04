// Credit to not-fl3/macroquad text: https://github.com/not-fl3/macroquad/blob/0.3/src/text.rs
use crate::rendering::*;
use crate::{Color, EmeraldError};

use miniquad::Context;
use std::collections::HashMap;

pub(crate) const DEFAULT_FONT_TEXTURE_PATH: &str = "ghosty_spooky_mister_mime_dude";

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FontKey(String);
impl FontKey {
    pub fn new<T: Into<String>>(font_path: T) -> Self {
        FontKey(font_path.into())
    }
}
impl Default for FontKey {
    fn default() -> FontKey {
        FontKey::new(DEFAULT_FONT_TEXTURE_PATH)
    }
}

pub(crate) struct CharacterInfo {
    pub offset_x: i32,
    pub offset_y: i32,
    pub advance: f32,

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
    pub inner: fontdue::Font,
    pub characters: HashMap<(char, u16), CharacterInfo>,
    pub font_texture: Texture,
    pub font_image: FontImage,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub max_line_height: u16,
}
impl Font {
    const GAP: u16 = 2;

    pub fn from_bytes(ctx: &mut miniquad::Context, bytes: &[u8]) -> Result<Font, EmeraldError> {
        let font_image = FontImage::gen_image_color(512, 512, Color::new(0, 0, 0, 0));
        let font_texture =
            Texture::from_rgba8(ctx, font_image.width, font_image.height, &font_image.bytes)?;

        Ok(Font {
            inner: fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())?,
            font_image,
            font_texture,
            characters: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            max_line_height: 0,
        })
    }

    pub fn cache_glyph(
        &mut self,
        mut ctx: &mut Context,
        character: char,
        size: u16,
    ) -> Result<(), EmeraldError> {
        let (metrics, bitmap) = self.inner.rasterize(character, size as f32);

        if metrics.advance_height != 0.0 {
            return Err(EmeraldError::new("Vertical fonts are not supported"));
        }

        let (width, height) = (metrics.width, metrics.height);
        let advance = metrics.advance_width;
        let (offset_x, offset_y) = (metrics.xmin, metrics.ymin);

        let x = if self.cursor_x + (width as u16) < self.font_image.width {
            if height as u16 > self.max_line_height {
                self.max_line_height = height as u16;
            }
            let res = self.cursor_x;
            self.cursor_x += width as u16 + Self::GAP;
            res
        } else {
            self.cursor_y += self.max_line_height + Self::GAP;
            self.cursor_x = width as u16 + Self::GAP;
            self.max_line_height = height as u16;
            Self::GAP
        };

        let y = self.cursor_y;

        let character_info = CharacterInfo {
            glyph_x: x as _,
            glyph_y: y as _,
            glyph_w: width as _,
            glyph_h: height as _,

            advance,
            offset_x,
            offset_y,
        };

        self.characters.insert((character, size), character_info);

        // texture bounds exceeded
        if self.cursor_y + height as u16 > self.font_image.height {
            // reset glyph cache state
            let characters = self.characters.drain().collect::<Vec<_>>();
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.max_line_height = 0;

            // increase font texture size
            self.font_image = FontImage::gen_image_color(
                self.font_image.width * 2,
                self.font_image.height * 2,
                Color::new(0, 0, 0, 0),
            );

            self.font_texture = Texture::from_rgba8(
                ctx,
                self.font_image.width,
                self.font_image.height,
                &self.font_image.bytes[..],
            )?;

            // recache all previously cached symbols
            for ((character, size), _) in characters {
                self.cache_glyph(&mut ctx, character, size)?;
            }
        } else {
            for j in 0..height {
                for i in 0..width {
                    let coverage = bitmap[j * width + i];

                    self.font_image.set_pixel(
                        x as u32 + i as u32,
                        y as u32 + j as u32,
                        Color::new(255, 255, 255, coverage),
                    );
                }
            }

            self.font_texture.update(&mut ctx, &self.font_image);
        }

        Ok(())
    }
}
