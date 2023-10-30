use crate::*;
use crate::{font::FontKey, rendering::*};

pub use fontdue::layout::{HorizontalAlign, VerticalAlign, WrapStyle};

#[derive(Clone)]
pub struct Label {
    pub text: String,
    pub offset: Vector2<f32>,
    pub scale: f32,
    pub font_key: FontKey,
    pub font_size: u16,
    pub z_index: f32,
    pub color: Color,
    pub centered: bool,
    pub visible: bool,
    pub visible_characters: i64,

    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
    pub wrap_style: WrapStyle,
    pub max_height: Option<f32>,
    pub max_width: Option<f32>,
}
impl Label {
    pub fn new<T: Into<String>>(text: T, font_key: FontKey, font_size: u16) -> Self {
        Label {
            font_key,
            text: text.into(),
            font_size,
            offset: Vector2::new(0.0, 0.0),
            scale: 1.0,
            z_index: 0.0,
            centered: true,
            color: WHITE,
            visible: true,
            visible_characters: -1,
            horizontal_align: HorizontalAlign::Center,
            vertical_align: VerticalAlign::Middle,
            wrap_style: WrapStyle::Word,
            max_height: None,
            max_width: Some(300.0),
        }
    }

    pub fn is_text_fully_revealed(&self) -> bool {
        self.visible_characters >= self.text.len() as i64 - 1
    }

    pub fn reveal_all_characters(&mut self) {
        self.visible_characters = self.text.len() as i64
    }
}
