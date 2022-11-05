mod button_state;
mod components;
mod input_engine;
mod input_handler;
mod mouse_state;
mod systems;
mod touch_state;

pub use button_state::*;
pub use components::*;
pub(crate) use input_engine::*;
pub use input_handler::*;
pub use mouse_state::*;
pub use systems::*;
pub use touch_state::*;

use crate::{transform::Translation, World};

/// Returns a world translation equivalent to the given point on a given screen.
pub fn screen_translation_to_world_translation(
    screen_size: (u32, u32),
    screen_translation: &Translation,
    world: &World,
) -> Translation {
    let camera_pos = world
        .get_active_camera()
        .and_then(|id| world.get::<&Translation>(id.clone()).ok())
        .map(|pos| *pos)
        .unwrap_or_default();

    // TODO(bombfuse): take the camera zoom level into account when translating
    let screen_size = Translation::new(screen_size.0 as f32, screen_size.1 as f32);
    let normalized_screen_translation = Translation::new(
        screen_translation.x - screen_size.x / 2.0,
        screen_size.y - screen_translation.y - screen_size.y / 2.0,
    );

    camera_pos + normalized_screen_translation
}

/// Describes touch-screen input state.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum MouseButton {
    Right,
    Left,
    Middle,
    Unknown,
}

#[derive(Debug, Copy, Clone)]
pub struct Touch {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum KeyCode {
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    RightShift,
    RightControl,
    RightAlt,
    Unknown,
}
