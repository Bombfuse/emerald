use crate::input::*;
use gilrs::*;
use miniquad::*;

use std::collections::HashMap;

pub struct InputEngine {
    gilrs: Gilrs,
    keys: HashMap<KeyCode, ButtonState>,
}
impl InputEngine {
    pub fn new() -> Self {
        InputEngine {
            gilrs: Gilrs::new().unwrap(),
            keys: HashMap::new(),
        }
    }

    pub fn update(&mut self) {
        for (key, mut state) in &mut self.keys {
            state.rollover();
        }

        while let Some(Event { id, event, time }) = self.gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
        }
    }

    pub fn get_key_state(&mut self, keycode: KeyCode) -> ButtonState {
        if let Some(key) = self.keys.get(&keycode) {
            return key.clone();
        }
        
        self.keys.insert(keycode, ButtonState::new());
        return self.get_key_state(keycode);
    }

    pub fn key_down(&mut self, keycode: KeyCode) {
        self.set_key_pressed(keycode, true)
    }

    pub fn key_up(&mut self, keycode: KeyCode) {
        self.set_key_pressed(keycode, false)
    }

    fn set_key_pressed(&mut self, keycode: KeyCode, is_pressed: bool) {
        if let Some(mut key) = self.keys.get_mut(&keycode) {
            key.is_pressed = is_pressed;
        } else {
            self.keys.insert(keycode, ButtonState::new());
            self.set_key_pressed(keycode, is_pressed);
        }
    }
}