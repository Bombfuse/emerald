use crate::input::*;
use crate::mouse_state::MouseState;
use miniquad::*;
use std::collections::HashMap;

#[cfg(feature = "gamepads")]
use gamepad::{Button, GamepadState, Joystick};

#[derive(Clone, Debug)]
pub struct InputHandler {
    keys: HashMap<KeyCode, ButtonState>,
    mouse: MouseState,
    #[cfg(feature = "gamepads")]
    gamepads: Vec<GamepadState>,
}
impl InputHandler {
    pub(crate) fn new(engine: &InputEngine) -> Self {
        InputHandler {
            keys: engine.keys.clone(),
            mouse: engine.mouse,
            #[cfg(feature = "gamepads")]
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
        self.get_key_state(key).is_just_pressed()
    }

    #[inline]
    pub fn get_key_state(&mut self, keycode: KeyCode) -> ButtonState {
        if let Some(key) = self.keys.get(&keycode) {
            return key.clone();
        }

        self.keys.insert(keycode, ButtonState::new());
        return self.get_key_state(keycode);
    }

    #[inline]
    pub fn mouse(&self) -> MouseState {
        self.mouse
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn is_button_pressed(&mut self, button: Button) -> bool {
        if let Some(gamepad) = self.gamepads.get(0) {
            return gamepad.is_pressed(button);
        }

        false
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn is_button_just_pressed(&mut self, button: Button) -> bool {
        if let Some(gamepad) = self.gamepads.get(0) {
            return gamepad.is_just_pressed(button);
        }

        false
    }
    
    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn is_button_pressed_for(&mut self, button: Button, index: usize) -> bool {
        if let Some(gamepad) = self.gamepads.get(index) {
            return gamepad.is_pressed(button);
        }

        false
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn is_button_just_pressed_for(&mut self, button: Button, index: usize) -> bool {
        if let Some(gamepad) = self.gamepads.get(index) {
            return gamepad.is_just_pressed(button);
        }

        false
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0.0, 0.0).
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn joystick(&mut self, joystick: Joystick) -> (f32, f32) {
        if let Some(gamepad) = self.gamepads.get(0) {
            return gamepad.joystick(joystick);
        }

        (0.0, 0.0)
    }
    /// Gets joystick value assuming first gamepad, defaulting to (0.0, 0.0).
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn joystick_for(&mut self, joystick: Joystick, index: usize) -> (f32, f32) {
        if let Some(gamepad) = self.gamepads.get(index) {
            return gamepad.joystick(joystick);
        }

        (0.0, 0.0)
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0, 0)
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn joystick_raw(&mut self, joystick: Joystick) -> (i16, i16) {
        if let Some(gamepad) = self.gamepads.get(0) {
            return gamepad.joystick_raw(joystick);
        }

        (0, 0)
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0, 0)
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn joystick_raw_for(&mut self, joystick: Joystick, index: usize) -> (i16, i16) {
        if let Some(gamepad) = self.gamepads.get(index) {
            return gamepad.joystick_raw(joystick);
        }

        (0, 0)
    }
}
