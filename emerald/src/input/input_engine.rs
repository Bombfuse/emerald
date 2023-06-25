use rapier2d::na::Vector2;

use crate::{input::*, AssetEngine, EmeraldError};

use std::collections::{HashMap, HashSet};

pub struct Action {
    pub key_bindings: HashSet<KeyCode>,
    pub button_bindings: HashMap<usize, HashSet<Button>>,
}
impl Action {
    pub fn new() -> Self {
        Self {
            key_bindings: HashSet::new(),
            button_bindings: HashMap::new(),
        }
    }
}

pub type ActionId = str;

pub trait InputEngine {
    fn initialize(&mut self, asset_engine: &mut AssetEngine);

    fn is_action_just_pressed(&mut self, action_label: &str) -> bool;
    fn is_action_pressed(&mut self, action_label: &str) -> bool;

    fn is_key_just_pressed(&mut self, key: KeyCode) -> bool;
    fn is_key_pressed(&mut self, key: KeyCode) -> bool;

    fn is_button_just_pressed(&mut self, button: Button, index: u8) -> bool;
    fn is_button_pressed(&mut self, button: Button, index: u8) -> bool;

    fn joystick(&mut self, joystick: Joystick, index: u8) -> Vector2<f32>;
    fn joystick_raw(&mut self, joystick: Joystick, index: u8) -> Vector2<f32>;

    fn add_action(&mut self, action_label: &str, action: Action);
    fn add_action_key(&mut self, action_label: &str, key_code: KeyCode);
    fn add_action_button(&mut self, action_label: &str, button: Button);
    fn remove_action_key(&mut self, action_label: &str, key_code: KeyCode);
    fn remove_action_button(&mut self, action_label: &str, button: Button);
    fn remove_action(&mut self, action_label: &str) -> Option<Action>;

    fn key_states_mut(&mut self) -> &mut HashMap<KeyCode, ButtonState>;
    fn button_states_mut(&mut self) -> &mut HashMap<Button, ButtonState>;

    fn update_and_rollover(&mut self) {
        for (_, state) in self.key_states_mut() {
            state.rollover();
        }
        for (_, state) in self.button_states_mut() {
            state.rollover();
        }
    }
}

// pub(crate) struct InputEngine {
//     #[cfg(feature = "gamepads")]
//     gamepad_engine: GamepadEngine,
//     #[cfg(feature = "gamepads")]
//     pub(crate) gamepads: Vec<GamepadState>,
//     pub(crate) keys: HashMap<KeyCode, ButtonState>,
//     pub(crate) mouse: MouseState,
//     pub(crate) touches: HashMap<u64, TouchState>,
//     pub(crate) touches_to_mouse: bool,
//     pub(crate) mouse_to_touch: bool,
//     pub(crate) actions: HashMap<ActionId, Action>,
// }
// impl InputEngine {
//     pub(crate) fn new() -> Self {
//         InputEngine {
//             #[cfg(feature = "gamepads")]
//             gamepad_engine: GamepadEngine::new(),
//             #[cfg(feature = "gamepads")]
//             gamepads: Vec::new(),
//             keys: HashMap::new(),
//             mouse: MouseState::new(),
//             touches: HashMap::new(),
//             touches_to_mouse: false,
//             mouse_to_touch: false,

//             actions: HashMap::new(),
//         }
//     }

//     pub fn handle_virtual_keycode(&mut self, virtual_keycode: VirtualKeyCode, state: ElementState) {
//         let keycode = virtual_keycode_to_keycode(virtual_keycode);

//         match state {
//             ElementState::Pressed => self.set_key_pressed(keycode, true),
//             ElementState::Released => self.set_key_pressed(keycode, false),
//         }
//     }

//     pub fn handle_cursor_move(&mut self, position: &winit::dpi::PhysicalPosition<f64>) {
//         self.mouse.translation = Translation::new(position.x as f32, position.y as f32);
//     }

//     pub fn handle_mouse_input(&mut self, button: &winit::event::MouseButton, state: &ElementState) {
//         let is_pressed = match state {
//             ElementState::Pressed => true,
//             ElementState::Released => false,
//         };

//         match button {
//             winit::event::MouseButton::Left => {
//                 self.mouse.left.is_pressed = is_pressed;
//             }
//             winit::event::MouseButton::Right => {
//                 self.mouse.right.is_pressed = is_pressed;
//             }
//             winit::event::MouseButton::Middle => {
//                 self.mouse.middle.is_pressed = is_pressed;
//             }
//             _ => {}
//         }
//     }

//     fn rollover_touches(&mut self) {
//         // Remove outdated touches.
//         self.touches.retain(|_id, touch| !touch.is_outdated());

//         // Then rollover the rest
//         for touch in self.touches.values_mut() {
//             touch.rollover();
//         }
//     }

//     #[inline]
//     #[cfg(feature = "gamepads")]
//     pub fn update_and_rollover(&mut self) -> Result<(), EmeraldError> {
//         self.gamepad_engine.update()?;
//         self.gamepads = self.gamepad_engine.gamepads().clone();

//         for state in self.keys.values_mut() {
//             state.rollover();
//         }
//         self.rollover_touches();
//         self.mouse.rollover();

//         Ok(())
//     }

//     #[inline]
//     #[cfg(not(feature = "gamepads"))]
//     pub fn update_and_rollover(&mut self) -> Result<(), EmeraldError> {
//         for (_key, state) in &mut self.keys {
//             state.rollover();
//         }
//         self.rollover_touches();
//         self.mouse.rollover();

//         Ok(())
//     }

//     fn add_action_if_not_exists(&mut self, action_id: &ActionId) {
//         if self.actions.contains_key(action_id) {
//             return;
//         }

//         self.actions.insert(action_id.clone(), Action::new());
//     }

//     #[inline]
//     pub fn add_action_binding_key(&mut self, action_id: &ActionId, key_code: KeyCode) {
//         self.add_action_if_not_exists(action_id);
//         if let Some(action) = self.actions.get_mut(action_id) {
//             action.key_bindings.insert(key_code);
//         }
//     }

//     #[inline]
//     #[cfg(feature = "gamepads")]
//     pub fn add_action_binding_button(
//         &mut self,
//         action_id: &ActionId,
//         button: Button,
//         gamepad_index: usize,
//     ) {
//         self.add_action_if_not_exists(action_id);

//         if let Some(action) = self.actions.get_mut(action_id) {
//             if !action.button_bindings.contains_key(&gamepad_index) {
//                 action.button_bindings.insert(gamepad_index, HashSet::new());
//             }

//             if let Some(set) = action.button_bindings.get_mut(&gamepad_index) {
//                 set.insert(button);
//             }
//         }
//     }

//     #[inline]
//     pub fn remove_action_binding_key(&mut self, action_id: &ActionId, key_code: &KeyCode) {
//         if let Some(action) = self.actions.get_mut(action_id) {
//             action.key_bindings.remove(key_code);
//         }
//     }

//     #[inline]
//     #[cfg(feature = "gamepads")]
//     pub fn remove_action_binding_button(
//         &mut self,
//         action_id: &ActionId,
//         button: &Button,
//         gamepad_index: usize,
//     ) {
//         if let Some(action) = self.actions.get_mut(action_id) {
//             if let Some(set) = action.button_bindings.get_mut(&gamepad_index) {
//                 set.remove(button);
//                 if set.len() == 0 {
//                     action.button_bindings.remove(&gamepad_index);
//                 }
//             }
//         }
//     }

//     #[inline]
//     pub fn set_key_down(&mut self, keycode: KeyCode, _repeat: bool) {
//         self.set_key_pressed(keycode, true)
//     }

//     #[inline]
//     pub fn set_key_up(&mut self, keycode: KeyCode) {
//         self.set_key_pressed(keycode, false)
//     }

//     #[inline]
//     fn set_key_pressed(&mut self, keycode: KeyCode, is_pressed: bool) {
//         if let Some(mut key) = self.keys.get_mut(&keycode) {
//             key.is_pressed = is_pressed;
//         } else {
//             self.keys.insert(keycode, ButtonState::new());
//             self.set_key_pressed(keycode, is_pressed);
//         }
//     }

//     #[inline]
//     pub fn set_mouse_translation(&mut self, x: f32, y: f32) {
//         self.mouse.translation = Translation::new(x, y);
//     }

//     #[inline]
//     pub fn set_mouse_down(&mut self, button: MouseButton, x: f32, y: f32) {
//         self.set_mouse_translation(x, y);
//         self.set_mouse_pressed(button, true);
//     }

//     #[inline]
//     pub fn set_mouse_up(&mut self, button: MouseButton, x: f32, y: f32) {
//         self.set_mouse_translation(x, y);
//         self.set_mouse_pressed(button, false);
//     }

//     #[inline]
//     fn set_mouse_pressed(&mut self, button: MouseButton, is_pressed: bool) {
//         let state = match button {
//             MouseButton::Right => &mut self.mouse.right,
//             MouseButton::Left => &mut self.mouse.left,
//             MouseButton::Middle => &mut self.mouse.middle,
//             MouseButton::Unknown => return,
//         };
//         state.is_pressed = is_pressed;
//     }

//     #[inline]
//     pub fn touch_event(&mut self, phase: TouchPhase, id: u64, x: f32, y: f32) {
//         let touch = self.touches.entry(id).or_default();
//         touch.translation = Translation::new(x, y);
//         touch.phase = phase;
//     }
// }

// pub(crate) fn virtual_keycode_to_keycode(virtual_keycode: VirtualKeyCode) -> KeyCode {
//     match virtual_keycode {
//         VirtualKeyCode::A => KeyCode::A,
//         VirtualKeyCode::B => KeyCode::B,
//         VirtualKeyCode::C => KeyCode::C,
//         VirtualKeyCode::D => KeyCode::D,
//         VirtualKeyCode::E => KeyCode::E,
//         VirtualKeyCode::F => KeyCode::F,
//         VirtualKeyCode::G => KeyCode::G,
//         VirtualKeyCode::H => KeyCode::H,
//         VirtualKeyCode::I => KeyCode::I,
//         VirtualKeyCode::J => KeyCode::J,
//         VirtualKeyCode::K => KeyCode::K,
//         VirtualKeyCode::L => KeyCode::L,
//         VirtualKeyCode::M => KeyCode::M,
//         VirtualKeyCode::N => KeyCode::N,
//         VirtualKeyCode::O => KeyCode::O,
//         VirtualKeyCode::P => KeyCode::P,
//         VirtualKeyCode::Q => KeyCode::Q,
//         VirtualKeyCode::R => KeyCode::R,
//         VirtualKeyCode::S => KeyCode::S,
//         VirtualKeyCode::T => KeyCode::T,
//         VirtualKeyCode::U => KeyCode::U,
//         VirtualKeyCode::V => KeyCode::V,
//         VirtualKeyCode::W => KeyCode::W,
//         VirtualKeyCode::X => KeyCode::X,
//         VirtualKeyCode::Y => KeyCode::Y,
//         VirtualKeyCode::Z => KeyCode::Z,
//         VirtualKeyCode::Down => KeyCode::Down,
//         VirtualKeyCode::Up => KeyCode::Up,
//         VirtualKeyCode::Left => KeyCode::Left,
//         VirtualKeyCode::Right => KeyCode::Right,
//         VirtualKeyCode::Space => KeyCode::Space,
//         VirtualKeyCode::Escape => KeyCode::Escape,
//         VirtualKeyCode::Delete => KeyCode::Delete,
//         VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
//         VirtualKeyCode::Comma => KeyCode::Comma,
//         VirtualKeyCode::Minus => KeyCode::Minus,
//         VirtualKeyCode::Period => KeyCode::Period,
//         VirtualKeyCode::Slash => KeyCode::Slash,
//         VirtualKeyCode::Key0 => KeyCode::Key0,
//         VirtualKeyCode::Key1 => KeyCode::Key1,
//         VirtualKeyCode::Key2 => KeyCode::Key2,
//         VirtualKeyCode::Key3 => KeyCode::Key3,
//         VirtualKeyCode::Key4 => KeyCode::Key4,
//         VirtualKeyCode::Key5 => KeyCode::Key5,
//         VirtualKeyCode::Key6 => KeyCode::Key6,
//         VirtualKeyCode::Key7 => KeyCode::Key7,
//         VirtualKeyCode::Key8 => KeyCode::Key8,
//         VirtualKeyCode::Key9 => KeyCode::Key9,
//         VirtualKeyCode::Semicolon => KeyCode::Semicolon,
//         VirtualKeyCode::Equals => KeyCode::Equal,
//         VirtualKeyCode::LBracket => KeyCode::LeftBracket,
//         VirtualKeyCode::Backslash => KeyCode::Backslash,
//         VirtualKeyCode::RBracket => KeyCode::RightBracket,
//         VirtualKeyCode::Grave => KeyCode::GraveAccent,
//         VirtualKeyCode::NumpadEnter => KeyCode::KpEnter,
//         VirtualKeyCode::Return => KeyCode::Enter,
//         VirtualKeyCode::Tab => KeyCode::Tab,
//         VirtualKeyCode::Back => KeyCode::Backspace,
//         VirtualKeyCode::Insert => KeyCode::Insert,
//         VirtualKeyCode::PageUp => KeyCode::PageUp,
//         VirtualKeyCode::PageDown => KeyCode::PageDown,
//         VirtualKeyCode::Home => KeyCode::Home,
//         VirtualKeyCode::End => KeyCode::End,
//         VirtualKeyCode::Capital => KeyCode::CapsLock,
//         VirtualKeyCode::Scroll => KeyCode::ScrollLock,
//         VirtualKeyCode::Numlock => KeyCode::NumLock,
//         VirtualKeyCode::Pause => KeyCode::Pause,
//         VirtualKeyCode::F1 => KeyCode::F1,
//         VirtualKeyCode::F2 => KeyCode::F2,
//         VirtualKeyCode::F3 => KeyCode::F3,
//         VirtualKeyCode::F4 => KeyCode::F4,
//         VirtualKeyCode::F5 => KeyCode::F5,
//         VirtualKeyCode::F6 => KeyCode::F6,
//         VirtualKeyCode::F7 => KeyCode::F7,
//         VirtualKeyCode::F8 => KeyCode::F8,
//         VirtualKeyCode::F9 => KeyCode::F9,
//         VirtualKeyCode::F10 => KeyCode::F10,
//         VirtualKeyCode::F11 => KeyCode::F11,
//         VirtualKeyCode::F12 => KeyCode::F12,
//         VirtualKeyCode::F13 => KeyCode::F13,
//         VirtualKeyCode::F14 => KeyCode::F14,
//         VirtualKeyCode::F15 => KeyCode::F15,
//         VirtualKeyCode::F16 => KeyCode::F16,
//         VirtualKeyCode::F17 => KeyCode::F17,
//         VirtualKeyCode::F18 => KeyCode::F18,
//         VirtualKeyCode::F19 => KeyCode::F19,
//         VirtualKeyCode::F20 => KeyCode::F20,
//         VirtualKeyCode::F21 => KeyCode::F21,
//         VirtualKeyCode::F22 => KeyCode::F22,
//         VirtualKeyCode::F23 => KeyCode::F23,
//         VirtualKeyCode::F24 => KeyCode::F24,
//         VirtualKeyCode::Numpad0 => KeyCode::Kp0,
//         VirtualKeyCode::Numpad1 => KeyCode::Kp1,
//         VirtualKeyCode::Numpad2 => KeyCode::Kp2,
//         VirtualKeyCode::Numpad3 => KeyCode::Kp3,
//         VirtualKeyCode::Numpad4 => KeyCode::Kp4,
//         VirtualKeyCode::Numpad5 => KeyCode::Kp5,
//         VirtualKeyCode::Numpad6 => KeyCode::Kp6,
//         VirtualKeyCode::Numpad7 => KeyCode::Kp7,
//         VirtualKeyCode::Numpad8 => KeyCode::Kp8,
//         VirtualKeyCode::Numpad9 => KeyCode::Kp9,
//         VirtualKeyCode::NumpadDecimal => KeyCode::KpDecimal,
//         VirtualKeyCode::NumpadDivide => KeyCode::KpDivide,
//         VirtualKeyCode::NumpadMultiply => KeyCode::KpMultiply,
//         VirtualKeyCode::NumpadSubtract => KeyCode::KpSubtract,
//         VirtualKeyCode::NumpadAdd => KeyCode::KpAdd,
//         VirtualKeyCode::NumpadEquals => KeyCode::KpEqual,
//         VirtualKeyCode::LShift => KeyCode::LeftShift,
//         VirtualKeyCode::LControl => KeyCode::LeftControl,
//         VirtualKeyCode::LAlt => KeyCode::LeftAlt,
//         VirtualKeyCode::RShift => KeyCode::RightShift,
//         VirtualKeyCode::RControl => KeyCode::RightControl,
//         VirtualKeyCode::RAlt => KeyCode::RightAlt,
//         _ => KeyCode::Unknown,
//     }
// }
