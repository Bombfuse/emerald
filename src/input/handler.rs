use crate::input::*;
use gamepad::{Button, GamepadState, Joystick};
use miniquad::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct InputHandler {
    keys: HashMap<KeyCode, ButtonState>,
    gamepads: Vec<GamepadState>,
}
impl InputHandler {
    pub(crate) fn new(engine: &InputEngine) -> Self {
        InputHandler {
            keys: engine.keys.clone(),
            gamepads: engine.gamepads.clone(),
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

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    pub fn is_button_pressed(&mut self, button: Button) -> bool {
        if let Some(gamepad) = self.gamepads.get(0) {
            return gamepad.is_pressed(button);
        }

        false
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    pub fn is_button_just_pressed(&mut self, button: Button) -> bool {
        if let Some(gamepad) = self.gamepads.get(0) {
            return gamepad.is_just_pressed(button);
        }

        false
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0.0, 0.0).
    #[inline]
    pub fn joystick(&mut self, joystick: Joystick) -> (f32, f32) {
        if let Some(gamepad) = self.gamepads.get(0) {
            return gamepad.joystick(joystick);
        }

        (0.0, 0.0)
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0, 0)
    #[inline]
    pub fn joystick_raw(&mut self, joystick: Joystick) -> (i16, i16) {
        if let Some(gamepad) = self.gamepads.get(0) {
            return gamepad.joystick_raw(joystick);
        }

        (0, 0)
    }
}
