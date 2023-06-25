use std::collections::HashMap;

use emerald::{AssetEngine, Button, ButtonState, InputEngine, KeyCode};

pub struct DesktopInputEngine {
    key_states: HashMap<KeyCode, ButtonState>,
    button_states: HashMap<Button, ButtonState>,
}
impl DesktopInputEngine {
    pub fn new() -> Self {
        Self {
            key_states: HashMap::new(),
            button_states: HashMap::new(),
        }
    }
}

impl InputEngine for DesktopInputEngine {
    fn initialize(&mut self, asset_engine: &mut AssetEngine) {}

    fn is_action_just_pressed(&mut self, action_label: &str) -> bool {
        todo!()
    }

    fn is_action_pressed(&mut self, action_label: &str) -> bool {
        todo!()
    }

    fn is_key_just_pressed(&mut self, key: emerald::KeyCode) -> bool {
        todo!()
    }

    fn is_key_pressed(&mut self, key: emerald::KeyCode) -> bool {
        todo!()
    }

    fn is_button_just_pressed(&mut self, button: emerald::Button, index: u8) -> bool {
        todo!()
    }

    fn is_button_pressed(&mut self, button: emerald::Button, index: u8) -> bool {
        todo!()
    }

    fn joystick(&mut self, joystick: emerald::Joystick, index: u8) -> emerald::Vector2<f32> {
        todo!()
    }

    fn joystick_raw(&mut self, joystick: emerald::Joystick, index: u8) -> emerald::Vector2<f32> {
        todo!()
    }

    fn add_action(&mut self, action_label: &str, action: emerald::Action) {
        todo!()
    }

    fn add_action_key(&mut self, action_label: &str, key_code: emerald::KeyCode) {
        todo!()
    }

    fn add_action_button(&mut self, action_label: &str, button: emerald::Button) {
        todo!()
    }

    fn remove_action_key(&mut self, action_label: &str, key_code: emerald::KeyCode) {
        todo!()
    }

    fn remove_action_button(&mut self, action_label: &str, button: emerald::Button) {
        todo!()
    }

    fn remove_action(&mut self, action_label: &str) -> Option<emerald::Action> {
        todo!()
    }

    fn key_states_mut(
        &mut self,
    ) -> &mut std::collections::HashMap<emerald::KeyCode, emerald::ButtonState> {
        &mut self.key_states
    }

    fn button_states_mut(
        &mut self,
    ) -> &mut std::collections::HashMap<emerald::Button, emerald::ButtonState> {
        &mut self.button_states
    }
}
