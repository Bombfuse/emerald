use std::collections::HashMap;

use emerald::{AssetEngine, Button, ButtonState, InputEngine, KeyCode, MouseState, Translation};

pub struct DesktopInputEngine {
    key_states: HashMap<KeyCode, ButtonState>,
    controller_states: HashMap<u8, HashMap<Button, ButtonState>>,
    mouse: MouseState,
}
impl DesktopInputEngine {
    pub fn new() -> Self {
        Self {
            key_states: HashMap::new(),
            controller_states: HashMap::new(),
            mouse: MouseState::new(),
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
        self.key_states
            .get(&key)
            .map(|state| state.is_just_pressed())
            .unwrap_or(false)
    }

    fn is_key_pressed(&mut self, key: emerald::KeyCode) -> bool {
        self.key_states
            .get(&key)
            .map(|state| state.is_pressed)
            .unwrap_or(false)
    }

    fn is_button_just_pressed(&mut self, button: emerald::Button, index: u8) -> bool {
        self.controller_states
            .get(&index)
            .map(|buttons| {
                buttons
                    .get(&button)
                    .map(|state| state.is_just_pressed())
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    fn is_button_pressed(&mut self, button: emerald::Button, index: u8) -> bool {
        self.controller_states
            .get(&index)
            .map(|buttons| {
                buttons
                    .get(&button)
                    .map(|state| state.is_pressed)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
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

    fn controller_states_mut(&mut self) -> &mut HashMap<u8, HashMap<Button, ButtonState>> {
        &mut self.controller_states
    }

    fn mouse(&self) -> MouseState {
        self.mouse.clone()
    }

    fn handle_cursor_move(&mut self, new_position: emerald::Vector2<f32>) {
        self.mouse.translation.x = new_position.x;
        self.mouse.translation.y = new_position.y;
    }

    fn handle_mouse_input(&mut self, button: emerald::MouseButton, is_pressed: bool) {
        match button {
            emerald::MouseButton::Right => self.mouse.right.is_pressed = is_pressed,
            emerald::MouseButton::Left => self.mouse.left.is_pressed = is_pressed,
            emerald::MouseButton::Middle => self.mouse.middle.is_pressed = is_pressed,
            emerald::MouseButton::Other(_) => {}
        }
    }

    fn handle_key_input(&mut self, key_code: KeyCode) {
        if !self.key_states.contains_key(&key_code) {
            self.key_states.insert(key_code, ButtonState::new());
        }

        self.key_states
            .get_mut(&key_code)
            .map(|button| button.is_pressed = true);
    }
}
