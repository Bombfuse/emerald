use crate::*;
use crate::rendering::{FontKey};

use miniquad_text_fontdue as quad_text;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Label {
    pub(crate) text: String,
    pub offset: Vector2<f32>,
    pub(crate) text_display_key: TextDisplayKey,
    pub(crate) is_text_up_to_date: bool,
}
impl Label {
    pub(crate) fn new(text_display_key: TextDisplayKey) -> Self {
        Label {
            text: String::from(""),
            offset: Vector2::new(0.0, 0.0),
            text_display_key,
            is_text_up_to_date: true,
        }
    }

    pub fn set_text<T: Into<String>>(&mut self, text: T) {
        self.text = text.into();
        self.is_text_up_to_date = false;
    }

    pub fn set_font() {}
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct TextDisplayKey(pub usize);
