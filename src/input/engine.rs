use crate::{input::*, EmeraldError};

#[cfg(feature = "gamepads")]
use gamepad::{GamepadEngine, GamepadState};

use miniquad::*;
use std::collections::HashMap;

#[cfg(not(feature = "gamepads"))]
pub(crate) struct InputEngine {
    pub(crate) keys: HashMap<KeyCode, ButtonState>,
}

#[cfg(feature = "gamepads")]
pub(crate) struct InputEngine {
    gamepad_engine: GamepadEngine,
    pub(crate) gamepads: Vec<GamepadState>,
    pub(crate) keys: HashMap<KeyCode, ButtonState>,
}
impl InputEngine {
    #[cfg(feature = "gamepads")]
    pub(crate) fn new() -> Self {
        InputEngine {
            gamepad_engine: GamepadEngine::new(),
            gamepads: Vec::new(),
            keys: HashMap::new(),
        }
    }

    #[cfg(not(feature = "gamepads"))]
    pub(crate) fn new() -> Self {
        InputEngine {
            keys: HashMap::new(),
        }
    }

    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn update_and_rollover(&mut self) -> Result<(), EmeraldError> {
        self.gamepad_engine.update()?;
        self.gamepads = self.gamepad_engine.gamepads().clone();

        for (_key, state) in &mut self.keys {
            state.rollover();
        }

        Ok(())
    }

    #[inline]
    #[cfg(not(feature = "gamepads"))]
    pub fn update_and_rollover(&mut self) -> Result<(), EmeraldError> {
        for (_key, state) in &mut self.keys {
            state.rollover();
        }

        Ok(())
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
