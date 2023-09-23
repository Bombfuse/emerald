use std::collections::HashMap;

use emerald::{
    Action, ActionId, AssetEngine, Button, ButtonState, InputEngine, KeyCode, KeyState, MouseState,
};

pub struct DesktopInputEngine {
    key_states: HashMap<KeyCode, ButtonState>,
    controller_states: HashMap<u8, HashMap<Button, ButtonState>>,
    mouse: MouseState,
    actions: HashMap<String, Action>,
}
impl DesktopInputEngine {
    pub fn new() -> Self {
        Self {
            key_states: HashMap::new(),
            controller_states: HashMap::new(),
            mouse: MouseState::new(),
            actions: HashMap::new(),
        }
    }

    fn get_action_mut(&mut self, action_id: &ActionId) -> &mut Action {
        if !self.actions.contains_key(action_id) {
            self.actions.insert(action_id.to_string(), Action::new());
        }

        self.actions.get_mut(action_id).unwrap()
    }
}

type ControllerStates = HashMap<u8, HashMap<Button, ButtonState>>;
fn is_key_just_pressed(key_states: &HashMap<KeyCode, ButtonState>, key: &KeyCode) -> bool {
    key_states
        .get(&key)
        .map(|state| state.is_just_pressed())
        .unwrap_or(false)
}
fn is_key_pressed(key_states: &HashMap<KeyCode, ButtonState>, key: &KeyCode) -> bool {
    key_states
        .get(&key)
        .map(|state| state.is_pressed)
        .unwrap_or(false)
}

fn is_button_just_pressed(
    controller_states: &ControllerStates,
    index: u8,
    button: &Button,
) -> bool {
    controller_states
        .get(&index)
        .map(|c| c.get(&button).map(|b| b.is_just_pressed()))
        .flatten()
        .unwrap_or(false)
}

fn is_button_pressed(controller_states: &ControllerStates, index: u8, button: &Button) -> bool {
    controller_states
        .get(&index)
        .map(|c| c.get(&button).map(|b| b.is_pressed))
        .flatten()
        .unwrap_or(false)
}

impl InputEngine for DesktopInputEngine {
    fn initialize(&mut self, _asset_engine: &mut AssetEngine) {}

    fn is_action_just_pressed(&mut self, action_label: &str) -> bool {
        self.actions
            .get(action_label)
            .map(|action| {
                for key in &action.key_bindings {
                    if is_key_just_pressed(&self.key_states, key) {
                        return true;
                    }
                }

                for (index, buttons) in &action.button_bindings {
                    for button in buttons {
                        if is_button_just_pressed(&self.controller_states, *index as u8, &button) {
                            return true;
                        }
                    }
                }

                false
            })
            .unwrap_or(false)
    }

    fn is_action_pressed(&mut self, action_label: &str) -> bool {
        self.actions
            .get(action_label)
            .map(|action| {
                for key in &action.key_bindings {
                    if is_key_pressed(&self.key_states, key) {
                        return true;
                    }
                }

                for (index, buttons) in &action.button_bindings {
                    for button in buttons {
                        if is_button_pressed(&self.controller_states, *index as u8, &button) {
                            return true;
                        }
                    }
                }

                false
            })
            .unwrap_or(false)
    }

    fn is_key_just_pressed(&mut self, key: emerald::KeyCode) -> bool {
        is_key_just_pressed(&self.key_states, &key)
    }

    fn is_key_pressed(&mut self, key: emerald::KeyCode) -> bool {
        is_key_pressed(&self.key_states, &key)
    }

    fn is_button_just_pressed(&mut self, button: emerald::Button, index: u8) -> bool {
        is_button_just_pressed(&self.controller_states, index, &button)
    }

    fn is_button_pressed(&mut self, button: emerald::Button, index: u8) -> bool {
        is_button_pressed(&self.controller_states, index, &button)
    }

    fn joystick(&mut self, _joystick: emerald::Joystick, _index: u8) -> emerald::Vector2<f32> {
        todo!()
    }

    fn joystick_raw(&mut self, _joystick: emerald::Joystick, _index: u8) -> emerald::Vector2<f32> {
        todo!()
    }

    fn add_action(&mut self, action_label: &str, action: emerald::Action) {
        self.actions.insert(action_label.to_string(), action);
    }

    fn add_action_key(&mut self, action_label: &str, key_code: emerald::KeyCode) {
        self.get_action_mut(action_label).add_key(key_code);
    }

    fn add_action_button(
        &mut self,
        action_label: &str,
        button: emerald::Button,
        gamepad_index: usize,
    ) {
        self.get_action_mut(action_label)
            .add_button(gamepad_index, button);
    }

    fn remove_action_key(&mut self, _action_label: &str, _key_code: emerald::KeyCode) {
        todo!()
    }

    fn remove_action_button(
        &mut self,
        action_label: &str,
        button: emerald::Button,
        gamepad_index: usize,
    ) {
        self.actions
            .get_mut(action_label)
            .map(|action| action.remove_button(gamepad_index, button));
    }

    fn remove_action(&mut self, _action_label: &str) -> Option<emerald::Action> {
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

    fn handle_key_input(&mut self, key_code: KeyCode, state: KeyState) {
        if !self.key_states.contains_key(&key_code) {
            self.key_states.insert(key_code, ButtonState::new());
        }

        self.key_states.get_mut(&key_code).map(|button| {
            button.is_pressed = match state {
                KeyState::Pressed => true,
                KeyState::Released => false,
            }
        });
    }
}
