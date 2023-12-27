use rapier2d::na::Vector2;

use crate::input::*;
use std::collections::HashMap;

pub struct InputHandler<'a> {
    engine: &'a mut Box<dyn InputEngine>,
}
impl<'a> InputHandler<'a> {
    pub(crate) fn new(engine: &'a mut Box<dyn InputEngine>) -> Self {
        Self { engine }
    }

    pub fn add_action_binding_keys(&mut self, action_id: &ActionId, key_codes: Vec<KeyCode>) {
        for key_code in key_codes {
            self.add_action_binding_key(action_id, key_code);
        }
    }

    #[inline]
    pub fn add_action_binding_key(&mut self, action_id: &ActionId, key_code: KeyCode) {
        self.engine.add_action_key(action_id, key_code)
    }

    #[inline]
    pub fn add_action_binding_button(
        &mut self,
        action_id: &ActionId,
        button: Button,
        gamepad_index: usize,
    ) {
        self.engine
            .add_action_button(action_id, button, gamepad_index)
    }

    #[inline]
    pub fn remove_action_binding_key(&mut self, action_id: &ActionId, key_code: &KeyCode) {
        // self.engine.remove_action_binding_key(action_id, key_code)
        todo!()
    }

    #[inline]
    pub fn remove_action_binding_button(
        &mut self,
        action_id: &ActionId,
        button: &Button,
        gamepad_index: usize,
    ) {
        // self.engine
        //     .remove_action_binding_button(action_id, button, gamepad_index)
        todo!()
    }

    #[inline]
    pub fn is_action_pressed(&mut self, action_id: &ActionId) -> bool {
        self.engine.is_action_pressed(action_id)
    }

    #[inline]
    pub fn is_action_just_pressed(&mut self, action_id: &ActionId) -> bool {
        self.engine.is_action_just_pressed(action_id)
    }

    #[inline]
    pub fn is_action_just_released(&mut self, action_id: &ActionId) -> bool {
        self.engine.is_action_just_released(action_id)
    }

    #[inline]
    pub fn is_key_pressed(&mut self, key: KeyCode) -> bool {
        self.engine.is_key_pressed(key)
    }

    #[inline]
    pub fn is_key_just_pressed(&mut self, key: KeyCode) -> bool {
        self.engine.is_key_just_pressed(key)
    }

    #[inline]
    pub fn mouse(&self) -> MouseState {
        self.engine.mouse().clone()
    }

    #[inline]
    pub fn touches(&self) -> &HashMap<u64, TouchState> {
        todo!()
    }

    /// Converts touches to mouse clicks
    pub fn touches_to_mouse(&mut self) {
        todo!()
    }

    /// Treat left mouse clicks as touches
    pub fn mouse_to_touch(&mut self) {
        todo!()
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    pub fn is_button_pressed(&mut self, button: Button) -> bool {
        self.engine.is_button_pressed(button, 0)
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    pub fn is_button_just_pressed(&mut self, button: Button) -> bool {
        self.engine.is_button_just_pressed(button, 0)
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    pub fn is_button_pressed_for(&mut self, button: Button, index: u8) -> bool {
        self.engine.is_button_pressed(button, index)
    }

    /// Gets the value of the button from the first gamepad available, or defaults to false.
    #[inline]
    pub fn is_button_just_pressed_for(&mut self, button: Button, index: u8) -> bool {
        self.engine.is_button_just_pressed(button, index)
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0.0, 0.0).
    #[inline]
    pub fn joystick(&mut self, joystick: Joystick) -> Vector2<f32> {
        self.engine.joystick(joystick, 0)
    }
    /// Gets joystick value assuming first gamepad, defaulting to (0.0, 0.0).
    #[inline]
    pub fn joystick_for(&mut self, joystick: Joystick, index: u8) -> Vector2<f32> {
        self.engine.joystick(joystick, index)
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0, 0)
    #[inline]
    pub fn joystick_raw(&mut self, joystick: Joystick) -> Vector2<f32> {
        self.engine.joystick_raw(joystick, 0)
    }

    /// Gets joystick value assuming first gamepad, defaulting to (0, 0)
    #[inline]
    pub fn joystick_raw_for(&mut self, joystick: Joystick, index: u8) -> Vector2<f32> {
        self.engine.joystick_raw(joystick, index)
    }
}
