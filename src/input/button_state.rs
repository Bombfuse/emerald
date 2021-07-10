#[derive(Copy, Clone, Debug, Default)]
pub struct ButtonState {
    pub is_pressed: bool,
    pub was_pressed: bool,
}
impl ButtonState {
    pub fn new() -> Self {
        ButtonState::default()
    }

    #[inline]
    pub(crate) fn rollover(&mut self) {
        self.was_pressed = self.is_pressed;
    }

    #[inline]
    pub fn is_just_pressed(&self) -> bool {
        !self.was_pressed && self.is_pressed
    }
}
