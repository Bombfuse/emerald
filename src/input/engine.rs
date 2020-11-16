use crate::input::*;
use miniquad::*;
use gamepad::{GamepadEngine, GamepadState};

use std::collections::HashMap;

pub(crate) struct InputEngine {
    gamepad_engine: GamepadEngine,
    pub(crate) keys: HashMap<KeyCode, ButtonState>,
    pub(crate) gamepads: Vec<GamepadState>,
}
impl InputEngine {
    pub(crate) fn new(gamepad_engine: GamepadEngine) -> Self {
        InputEngine {
            gamepad_engine,
            keys: HashMap::new(),
            gamepads: Vec::new(),
        }
    }

    #[inline]
    pub fn update_and_rollover(&mut self) {
        self.gamepad_engine.update();
        self.gamepads = self.gamepad_engine.gamepads().clone();

        for (_key, state) in &mut self.keys {
            state.rollover();
        }
    }

    #[inline]
    pub fn set_key_down(&mut self, keycode: KeyCode, _repeat: bool) {
        self.set_key_pressed(keycode, true)
    }

    #[inline]
    pub fn set_key_up(&mut self, keycode: KeyCode) {
        self.set_key_pressed(keycode, false)
    }

    #[inline]
    fn set_key_pressed(&mut self, keycode: KeyCode, is_pressed: bool) {
        if let Some(mut key) = self.keys.get_mut(&keycode) {
            key.is_pressed = is_pressed;
        } else {
            self.keys.insert(keycode, ButtonState::new());
            self.set_key_pressed(keycode, is_pressed);
        }
    }
}
