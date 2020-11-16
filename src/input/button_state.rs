#[derive(Copy, Clone, Debug)]
pub struct ButtonState {
    pub is_pressed: bool,
    pub was_pressed: bool,
}
impl ButtonState {
    pub fn new() -> Self {
        ButtonState {
            is_pressed: false,
            was_pressed: false,
        }
    }

    #[inline]
    pub(crate) fn rollover(&mut self) {
        self.was_pressed = self.is_pressed;
    }
}
