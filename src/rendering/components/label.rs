use crate::rendering::*;
use crate::*;

#[derive(Clone, Debug)]
pub struct Label {
    pub text: String,
    pub offset: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub font_key: FontKey,
    pub font_size: u16,
    pub z_index: f32,
    pub centered: bool,
}
impl Label {
    pub fn new<T: Into<String>>(text: T, font_key: FontKey) -> Self {
        let mut label = Label::default();
        label.font_key = font_key;
        label.text = text.into();

        label
    }
}
impl Default for Label {
    fn default() -> Label {
        Label {
            text: String::from(""),
            offset: Vector2::new(0.0, 0.0),
            font_key: FontKey::default(),
            scale: Vector2::new(1.0, 1.0),
            font_size: 12,
            z_index: 0.0,
            centered: true,
        }
    }
}
