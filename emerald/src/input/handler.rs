use crate::input::*;
use std::collections::HashMap;

#[cfg(feature = "gamepads")]
use gamepad::{Button, Joystick};

pub struct InputHandler<'a> {
    engine: &'a mut InputEngine,
}
impl<'a> InputHandler<'a> {
    pub(crate) fn new(engine: &'a mut InputEngine) -> Self {
        Self { engine }
    }

    #[inline]
    pub fn add_action_binding_key(&mut self, action_id: &ActionId, key_code: KeyCode) {
        self.engine.add_action_binding_key(action_id, key_code)
    }

    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn add_action_binding_button(
        &mut self,
        action_id: &ActionId,
        button: Button,
        gamepad_index: usize,
    ) {
        self.engine
            .add_action_binding_button(action_id, button, gamepad_index)
    }

    #[inline]
    pub fn remove_action_binding_key(&mut self, action_id: &ActionId, key_code: &KeyCode) {
        self.engine.remove_action_binding_key(action_id, key_code)
    }

    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn remove_action_binding_button(
        &mut self,
        action_id: &ActionId,
        button: &Button,
        gamepad_index: usize,
    ) {
        self.engine
            .remove_action_binding_button(action_id, button, gamepad_index)
    }

    #[inline]
    pub fn is_action_pressed(&mut self, action_id: &ActionId) -> bool {
        let mut keys = Vec::new();
        #[cfg(feature = "gamepads")]
        let mut buttons = Vec::new();

        if let Some(action) = self.engine.actions.get(action_id) {
            for key in &action.key_bindings {
                keys.push(*key);
            }

            #[cfg(feature = "gamepads")]
            {
                for (gamepad_index, set) in &action.button_bindings {
                    for button in set {
                        buttons.push((*gamepad_index, *button));
                    }
                }
            }
        }

        for key in keys {
            if self.is_key_pressed(key) {
                return true;
            }
        }

        #[cfg(feature = "gamepads")]
        {
            for (gamepad_index, button) in buttons {
                if self.is_button_pressed_for(button, gamepad_index) {
                    return true;
                }
            }
        }

        false
    }

    #[inline]
    pub fn is_action_just_pressed(&mut self, action_id: &ActionId) -> bool {
        let mut keys = Vec::new();
        #[cfg(feature = "gamepads")]
        let mut buttons = Vec::new();

        if let Some(action) = self.engine.actions.get(action_id) {
            for key in &action.key_bindings {
                keys.push(*key);
            }

            #[cfg(feature = "gamepads")]
            {
                for (gamepad_index, set) in &action.button_bindings {
                    for button in set {
                        buttons.push((*gamepad_index, *button));
                    }
                }
            }
        }

        for key in keys {
            if self.is_key_just_pressed(key) {
                return true;
            }
        }

        #[cfg(feature = "gamepads")]
        {
            for (gamepad_index, button) in buttons {
                if self.is_button_just_pressed_for(button, gamepad_index) {
                    return true;
                }
            }
        }

        false
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
        if let Some(key) = self.engine.keys.get(&keycode) {
            return *key;
        }

        self.engine.keys.insert(keycode, ButtonState::new());
        self.get_key_state(keycode)
    }

    #[inline]
    pub fn mouse(&self) -> MouseState {
        self.engine.mouse
    }

    #[inline]
    pub fn touches(&self) -> &HashMap<u64, TouchState> {
        &self.engine.touches
    }

    /// Converts touches to mouse clicks
    pub fn touches_to_mouse(&mut self) {
        let last_touch = self.engine.touches.values().last();
        let last_touch = match last_touch {
            Some(x) => x,
            None => return,
        };

        let number_of_touches = self.engine.touches.len();
        let button = match number_of_touches {
            1 => &mut self.engine.mouse.left,
            2 => &mut self.engine.mouse.right,
            _ => &mut self.engine.mouse.middle,
        };

        self.engine.mouse.translation = last_touch.translation;
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
        let (previous, phase) = match (
            self.engine.mouse.left.was_pressed,
            self.engine.mouse.left.is_pressed,
        ) {
            (false, false) => return,
            (false, true) => (TouchPhase::Cancelled, TouchPhase::Started),
            (true, true) => (TouchPhase::Moved, TouchPhase::Moved),
            (true, false) => (TouchPhase::Moved, TouchPhase::Ended),
        };

        // We need some fake id:
        let id = 0xC0FFEE;

        self.engine.touches.insert(
            id,
            TouchState {
                translation: self.engine.mouse.translation,
                previous,
                phase,
            },
        );
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn is_button_pressed(&mut self, button: Button) -> bool {
        if let Some(gamepad) = self.engine.gamepads.get(0) {
            return gamepad.is_pressed(button);
        }

        false
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn is_button_just_pressed(&mut self, button: Button) -> bool {
        if let Some(gamepad) = self.engine.gamepads.get(0) {
            return gamepad.is_just_pressed(button);
        }

        false
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn is_button_pressed_for(&mut self, button: Button, index: usize) -> bool {
        if let Some(gamepad) = self.engine.gamepads.get(index) {
            return gamepad.is_pressed(button);
        }

        false
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn is_button_just_pressed_for(&mut self, button: Button, index: usize) -> bool {
        if let Some(gamepad) = self.engine.gamepads.get(index) {
            return gamepad.is_just_pressed(button);
        }

        false
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0.0, 0.0).
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn joystick(&mut self, joystick: Joystick) -> (f32, f32) {
        if let Some(gamepad) = self.engine.gamepads.get(0) {
            return gamepad.joystick(joystick);
        }

        (0.0, 0.0)
    }
    /// Gets joystick value assuming first gamepad, defaulting to (0.0, 0.0).
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn joystick_for(&mut self, joystick: Joystick, index: usize) -> (f32, f32) {
        if let Some(gamepad) = self.engine.gamepads.get(index) {
            return gamepad.joystick(joystick);
        }

        (0.0, 0.0)
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0, 0)
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn joystick_raw(&mut self, joystick: Joystick) -> (i16, i16) {
        if let Some(gamepad) = self.engine.gamepads.get(0) {
            return gamepad.joystick_raw(joystick);
        }

        (0, 0)
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0, 0)
    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn joystick_raw_for(&mut self, joystick: Joystick, index: usize) -> (i16, i16) {
        if let Some(gamepad) = self.engine.gamepads.get(index) {
            return gamepad.joystick_raw(joystick);
        }

        (0, 0)
    }
}
