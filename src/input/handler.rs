use crate::input::*;
use miniquad::*;
use std::collections::HashMap;

#[cfg(feature = "gamepads")]
use gamepad::{Button, GamepadState, Joystick};

#[derive(Clone, Debug)]
pub struct InputHandler {
    keys: HashMap<KeyCode, ButtonState>,
    mouse: MouseState,
    touches: HashMap<u64, TouchState>,
    #[cfg(feature = "gamepads")]
    gamepads: Vec<GamepadState>,
}
impl InputHandler {
    pub(crate) fn new(engine: &InputEngine) -> Self {
        let mut this = InputHandler {
            keys: engine.keys.clone(),
            mouse: engine.mouse,
            touches: engine.touches.clone(),
            #[cfg(feature = "gamepads")]
            gamepads: engine.gamepads.clone(),
        };
        if engine.mouse_to_touch {
            this.mouse_to_touch();
        }
        if engine.touches_to_mouse {
            this.touches_to_mouse();
        }
        this
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

    #[inline]
    pub fn touches(&self) -> &HashMap<u64, TouchState> {
        &self.touches
    }

    /// Converts touches to mouse clicks
    pub fn touches_to_mouse(&mut self) {
        let last_touch = self.touches.values().last();
        let last_touch = match last_touch {
            Some(x) => x,
            None => return,
        };

        let number_of_touches = self.touches.len();
        let button = match number_of_touches {
            1 => &mut self.mouse.left,
            2 => &mut self.mouse.right,
            _ => &mut self.mouse.middle,
        };

        self.mouse.position = last_touch.position;
        match last_touch.phase {
            TouchPhase::Started => {
                button.was_pressed = false;
                button.is_pressed = true;
            }
            TouchPhase::Moved => {
                button.was_pressed = true;
                button.is_pressed = true;
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                button.was_pressed = true;
                button.is_pressed = false;
            }
        }
    }

    /// Treat left mouse clicks as touches
    pub fn mouse_to_touch(&mut self) {
        let (previous, phase) = match (self.mouse.left.was_pressed, self.mouse.left.is_pressed) {
            (false, false) => return,
            (false, true) => (TouchPhase::Cancelled, TouchPhase::Started),
            (true, true) => (TouchPhase::Moved, TouchPhase::Moved),
            (true, false) => (TouchPhase::Moved, TouchPhase::Ended),
        };

        // We need some fake id:
        let id = 0xC0FFEE;

        self.touches.insert(
            id,
            TouchState {
                position: self.mouse.position,
                previous,
                phase,
            },
        );
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
