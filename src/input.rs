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

use crate::{World, transform::Translation};


/// Returns a world translation equivalent to the given point on a given screen.
pub fn screen_translation_to_world_translation(
    screen_size: (u32, u32),
    screen_translation: &Translation,
    world: &mut World,
) -> Translation {
    let mut p = screen_translation.clone();
    let mut camera_pos = Translation::default();

    if let Some(id) = world.get_active_camera() {
        if let Ok(pos) = world.get_mut::<Translation>(id.clone()) {
            camera_pos.x = pos.x;
            camera_pos.y = pos.y;
        }
    }

    // TODO(bombfuse): take the camera zoom level into account when translating
    p.x = (screen_size.0 as f32) / -2.0 + screen_translation.x + camera_pos.x;
    p.y = (screen_size.1 as f32) / -2.0 + screen_translation.y + camera_pos.y;

    p
}
