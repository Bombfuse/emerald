use crate::*;
use crate::rendering::{FontKey};

#[derive(Clone, Debug)]
pub struct Label {
    pub text: String,
    pub offset: Vector2<f32>,
    pub font: FontKey,
}
impl Label {
    pub(crate) fn new(font: FontKey) -> Self {
        Label {
            text: String::from(""),
            offset: Vector2::new(0.0, 0.0),
            font,
        }
    }
}
impl Default for Label {
    fn default() -> Label {
        Label::new(FontKey::default())
    }
}