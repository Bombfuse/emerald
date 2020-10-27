use crate::*;
use crate::rendering::*;

#[derive(Clone, Debug)]
pub struct Label {
    pub text: String,
    pub offset: Vector2<f32>,
    pub font_key: FontKey,
}
impl Label {
    pub fn new<T: Into<String>>(text: T, font_key: FontKey) -> Self {
        Label {
            text: text.into(),
            offset: Vector2::new(0.0, 0.0),
            font_key,
        }
    }
}
impl Default for Label {
    fn default() -> Label {
        Label {
            text: String::from(""),
            offset: Vector2::new(0.0, 0.0),
            font_key: FontKey::default(),
        }
    }
}