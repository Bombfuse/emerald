use miniquad::*;
use crate::input::*;

pub struct InputHandler<'a> {
    engine: &'a mut InputEngine,
}
impl<'a> InputHandler<'a> {
    pub fn new(engine: &'a mut InputEngine) -> Self {
        InputHandler {
            engine,
        }
    }

    pub fn is_key_pressed(&mut self, key: KeyCode) -> bool {
        let key_state = self.engine.get_key_state(key);

        key_state.is_pressed
    }

    pub fn is_key_just_pressed(&mut self, key: KeyCode) -> bool {
        let key_state = self.engine.get_key_state(key);

        // println!("({}, {})", key_state.was_pressed, key_state.is_pressed);

        key_state.is_pressed && !key_state.was_pressed
    }
}