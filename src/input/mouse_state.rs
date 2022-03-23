use crate::{ButtonState, transform::Translation};

/// State of mouse over last few frames
#[derive(Clone, Copy, Debug, Default)]
pub struct MouseState {
    pub translation: Translation,
    pub left: ButtonState,
    pub middle: ButtonState,
    pub right: ButtonState,
}

impl MouseState {
    pub fn new() -> Self {
        MouseState::default()
    }

    pub(crate) fn rollover(&mut self) {
        self.left.rollover();
        self.middle.rollover();
        self.right.rollover();
    }
}
