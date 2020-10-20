use crate::input::*;
use miniquad::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct InputHandler {
    keys: HashMap<KeyCode, ButtonState>,
}
impl InputHandler {
    pub(crate) fn new(engine: &InputEngine) -> Self {
        InputHandler {
            keys: engine.keys.clone(),
        }
    }

    #[inline]
    pub fn is_key_pressed(&mut self, key: KeyCode) -> bool {
        let key_state = self.get_key_state(key);

        key_state.is_pressed
    }

    #[inline]
    pub fn is_key_just_pressed(&mut self, key: KeyCode) -> bool {
        let key_state = self.get_key_state(key);

        key_state.is_pressed && !key_state.was_pressed
    }

    #[inline]
    pub fn get_key_state(&mut self, keycode: KeyCode) -> ButtonState {
        if let Some(key) = self.keys.get(&keycode) {
            return key.clone();
        }

        self.keys.insert(keycode, ButtonState::new());
        return self.get_key_state(keycode);
    }
}
