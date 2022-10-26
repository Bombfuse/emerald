use crate::{input::*, EmeraldError};

#[cfg(feature = "gamepads")]
use gamepad::{Button, GamepadEngine, GamepadState};
use winit::event::{ElementState, VirtualKeyCode};

use std::collections::{HashMap, HashSet};

use super::touch_state::TouchState;

pub(crate) struct Action {
    pub key_bindings: HashSet<KeyCode>,
    #[cfg(feature = "gamepads")]
    pub button_bindings: HashMap<usize, HashSet<Button>>,
}
impl Action {
    pub fn new() -> Self {
        Self {
            key_bindings: HashSet::new(),
            #[cfg(feature = "gamepads")]
            button_bindings: HashMap::new(),
        }
    }
}

pub type ActionId = String;

pub(crate) struct InputEngine {
    #[cfg(feature = "gamepads")]
    gamepad_engine: GamepadEngine,
    #[cfg(feature = "gamepads")]
    pub(crate) gamepads: Vec<GamepadState>,
    pub(crate) keys: HashMap<KeyCode, ButtonState>,
    pub(crate) mouse: MouseState,
    pub(crate) touches: HashMap<u64, TouchState>,
    pub(crate) touches_to_mouse: bool,
    pub(crate) mouse_to_touch: bool,
    pub(crate) actions: HashMap<ActionId, Action>,
}
impl InputEngine {
    pub(crate) fn new() -> Self {
        InputEngine {
            #[cfg(feature = "gamepads")]
            gamepad_engine: GamepadEngine::new(),
            #[cfg(feature = "gamepads")]
            gamepads: Vec::new(),
            keys: HashMap::new(),
            mouse: MouseState::new(),
            touches: HashMap::new(),
            touches_to_mouse: false,
            mouse_to_touch: false,

            actions: HashMap::new(),
        }
    }

    pub fn handle_virtual_keycode(&mut self, virtual_keycode: VirtualKeyCode, state: ElementState) {
        let keycode = virtual_keycode_to_keycode(virtual_keycode);

        match state {
            ElementState::Pressed => self.set_key_pressed(keycode, true),
            ElementState::Released => self.set_key_pressed(keycode, false),
        }
    }

    pub fn handle_cursor_move(&mut self, position: &winit::dpi::PhysicalPosition<f64>) {
        self.mouse.translation = Translation::new(position.x as f32, position.y as f32);
    }

    pub fn handle_mouse_input(&mut self, button: &winit::event::MouseButton, state: &ElementState) {
        let is_pressed = match state {
            ElementState::Pressed => true,
            ElementState::Released => false,
        };

        match button {
            winit::event::MouseButton::Left => {
                self.mouse.left.is_pressed = is_pressed;
            }
            winit::event::MouseButton::Right => {
                self.mouse.right.is_pressed = is_pressed;
            }
            winit::event::MouseButton::Middle => {
                self.mouse.middle.is_pressed = is_pressed;
            }
            _ => {}
        }
    }

    fn rollover_touches(&mut self) {
        // Remove outdated touches.
        self.touches.retain(|_id, touch| !touch.is_outdated());

        // Then rollover the rest
        for touch in self.touches.values_mut() {
            touch.rollover();
        }
    }

    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn update_and_rollover(&mut self) -> Result<(), EmeraldError> {
        self.gamepad_engine.update()?;
        self.gamepads = self.gamepad_engine.gamepads().clone();

        for state in self.keys.values_mut() {
            state.rollover();
        }
        self.rollover_touches();
        self.mouse.rollover();

        Ok(())
    }

    #[inline]
    #[cfg(not(feature = "gamepads"))]
    pub fn update_and_rollover(&mut self) -> Result<(), EmeraldError> {
        for (_key, state) in &mut self.keys {
            state.rollover();
        }
        self.rollover_touches();
        self.mouse.rollover();

        Ok(())
    }

    fn add_action_if_not_exists(&mut self, action_id: &ActionId) {
        if self.actions.contains_key(action_id) {
            return;
        }

        self.actions.insert(action_id.clone(), Action::new());
    }

    #[inline]
    pub fn add_action_binding_key(&mut self, action_id: &ActionId, key_code: KeyCode) {
        self.add_action_if_not_exists(action_id);
        if let Some(action) = self.actions.get_mut(action_id) {
            action.key_bindings.insert(key_code);
        }
    }

    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn add_action_binding_button(
        &mut self,
        action_id: &ActionId,
        button: Button,
        gamepad_index: usize,
    ) {
        self.add_action_if_not_exists(action_id);

        if let Some(action) = self.actions.get_mut(action_id) {
            if !action.button_bindings.contains_key(&gamepad_index) {
                action.button_bindings.insert(gamepad_index, HashSet::new());
            }

            if let Some(set) = action.button_bindings.get_mut(&gamepad_index) {
                set.insert(button);
            }
        }
    }

    #[inline]
    pub fn remove_action_binding_key(&mut self, action_id: &ActionId, key_code: &KeyCode) {
        if let Some(action) = self.actions.get_mut(action_id) {
            action.key_bindings.remove(key_code);
        }
    }

    #[inline]
    #[cfg(feature = "gamepads")]
    pub fn remove_action_binding_button(
        &mut self,
        action_id: &ActionId,
        button: &Button,
        gamepad_index: usize,
    ) {
        if let Some(action) = self.actions.get_mut(action_id) {
            if let Some(set) = action.button_bindings.get_mut(&gamepad_index) {
                set.remove(button);
                if set.len() == 0 {
                    action.button_bindings.remove(&gamepad_index);
                }
            }
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

    #[inline]
    pub fn set_mouse_translation(&mut self, x: f32, y: f32) {
        self.mouse.translation = Translation::new(x, y);
    }

    #[inline]
    pub fn set_mouse_down(&mut self, button: MouseButton, x: f32, y: f32) {
        self.set_mouse_translation(x, y);
        self.set_mouse_pressed(button, true);
    }

    #[inline]
    pub fn set_mouse_up(&mut self, button: MouseButton, x: f32, y: f32) {
        self.set_mouse_translation(x, y);
        self.set_mouse_pressed(button, false);
    }

    #[inline]
    fn set_mouse_pressed(&mut self, button: MouseButton, is_pressed: bool) {
        let state = match button {
            MouseButton::Right => &mut self.mouse.right,
            MouseButton::Left => &mut self.mouse.left,
            MouseButton::Middle => &mut self.mouse.middle,
            MouseButton::Unknown => return,
        };
        state.is_pressed = is_pressed;
    }

    #[inline]
    pub fn touch_event(&mut self, phase: TouchPhase, id: u64, x: f32, y: f32) {
        let touch = self.touches.entry(id).or_default();
        touch.translation = Translation::new(x, y);
        touch.phase = phase;
    }
}

pub(crate) fn virtual_keycode_to_keycode(virtual_keycode: VirtualKeyCode) -> KeyCode {
    match virtual_keycode {
        VirtualKeyCode::A => KeyCode::A,
        VirtualKeyCode::S => KeyCode::S,
        VirtualKeyCode::Down => KeyCode::Down,
        VirtualKeyCode::Up => KeyCode::Up,
        VirtualKeyCode::Left => KeyCode::Left,
        VirtualKeyCode::Right => KeyCode::Right,
        VirtualKeyCode::Space => KeyCode::Space,
        _ => KeyCode::Unknown,
    }
}
