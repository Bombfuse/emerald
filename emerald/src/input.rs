mod button_state;
mod components;
mod engine;
mod handler;
mod mouse_state;
mod systems;
mod touch_state;

pub use button_state::*;
pub use components::*;
pub(crate) use engine::*;
pub use handler::*;
pub use miniquad::KeyCode;
pub use mouse_state::*;
pub use systems::*;
pub use touch_state::*;

use crate::{transform::Translation, World};

/// Returns a world translation equivalent to the given point on a given screen.
pub fn screen_translation_to_world_translation(
    screen_size: (u32, u32),
    screen_translation: &Translation,
    world: &mut World,
) -> Translation {
    let camera_pos = world
        .get_active_camera()
        .and_then(|id| world.get_mut::<Translation>(id.clone()).ok())
        .map(|pos| *pos)
        .unwrap_or_default();

    // TODO(bombfuse): take the camera zoom level into account when translating
    let screen_size = Translation::new(screen_size.0 as f32, screen_size.1 as f32);
    Translation::from(screen_size * -0.5) + *screen_translation + camera_pos
}
