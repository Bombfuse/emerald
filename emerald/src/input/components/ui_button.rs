use crate::{asset_key::AssetKey, Rectangle};

pub struct UIButton {
    pub pressed_texture: AssetKey,
    pub unpressed_texture: AssetKey,

    /// Custom bounding box for the pressed area of the button, overwrites the usage of the texture for the box.
    pub custom_pressed_bounding_box: Option<Rectangle>,

    /// Custom bounding box for the unpressed area of the button, overwrites the usage of the texture for the box.
    pub custom_unpressed_bounding_box: Option<Rectangle>,

    pub(crate) is_pressed: bool,
    pub(crate) was_pressed: bool,

    pub z_index: f32,
    pub visible: bool,
}
impl UIButton {
    pub fn new(pressed_texture: AssetKey, unpressed_texture: AssetKey) -> Self {
        UIButton {
            unpressed_texture,
            pressed_texture,
            custom_pressed_bounding_box: None,
            custom_unpressed_bounding_box: None,
            is_pressed: false,
            was_pressed: false,
            z_index: 0.0,
            visible: true,
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }
    pub fn is_just_pressed(&self) -> bool {
        self.is_pressed && !self.was_pressed
    }
    pub fn is_just_released(&self) -> bool {
        !self.is_pressed && self.was_pressed
    }

    /// Presses the button
    pub fn press(&mut self) {
        self.rollover();
        self.is_pressed = true;
    }

    /// Releases the button
    pub fn release(&mut self) {
        self.rollover();
        self.is_pressed = false;
    }

    fn rollover(&mut self) {
        self.was_pressed = self.is_pressed;
        self.is_pressed = false;
    }

    pub fn reset(&mut self) {
        self.is_pressed = false;
        self.was_pressed = false;
    }

    pub(crate) fn current_texture(&self) -> &AssetKey {
        if self.is_pressed() {
            &self.pressed_texture
        } else {
            &self.unpressed_texture
        }
    }
}
