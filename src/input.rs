mod button_state;
mod engine;
mod handler;
mod mouse_state;
mod touch_state;
mod components;
mod systems;

pub use button_state::*;
pub(crate) use engine::*;
pub use handler::*;
pub use miniquad::KeyCode;
pub use mouse_state::*;
pub use components::*;
pub use systems::*;
pub use touch_state::*;

use crate::{EmeraldWorld, Position};


pub fn screen_position_to_world_position(screen_size: (u32, u32), screen_position: &Position, world: &mut EmeraldWorld) -> Position {
    let mut p = screen_position.clone();
    // let mut camera = Camera::default();
    let mut camera_pos = Position::zero();

    if let Some(id) = world.get_active_camera() {
        if let Ok(pos) = world.get_mut::<Position>(id.clone()) {
            camera_pos.x = pos.x;
            camera_pos.y = pos.y;
        }

        // if let Ok(c) = world.get_mut::<Camera>(id) {
        //     camera = c.clone();
        // }
    }

    // TODO(bombfuse): take the camera zoom level into account when translating
    p.x = (screen_size.0 as f32) / -2.0 + screen_position.x + camera_pos.x;
    p.y = (screen_size.1 as f32) / -2.0 + screen_position.y + camera_pos.y;

    p
}