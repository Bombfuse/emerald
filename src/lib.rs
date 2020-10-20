mod assets;
mod audio;
mod colors;
mod core;
mod input;
mod logging;
mod rendering;
mod types;
mod world;

pub use crate::assets::*;
pub use crate::colors::*;
pub use crate::core::*;
pub use crate::input::*;
pub use crate::rendering::*;
pub use crate::types::*;
pub use crate::world::physics::*;
pub use crate::world::*;
pub use audio::*;
pub use logging::*;

// physics/math/collision re-exports
pub use rapier2d::dynamics::{RigidBody, RigidBodyBuilder, RigidBodyHandle};
pub use rapier2d::geometry::{Collider, ColliderBuilder, ColliderHandle};
pub use rapier2d::na::Vector2;
//

// General re-exports for compatibility
pub use hecs;
pub use nanoserde;
pub use quad_rand as random;
pub use quad_rand as rand;
pub use rapier2d;
pub use rapier2d::crossbeam;
pub use rapier2d::na as nalgebra;

pub use hecs::*;

use miniquad::{conf, UserData};

pub fn start(game: Box<dyn Game>, settings: GameSettings) {
    let mut config = conf::Conf::default();
    config.window_title = settings.title.clone();
    config.window_width = settings.render_settings.resolution.0 as i32;
    config.window_height = settings.render_settings.resolution.1 as i32;
    config.fullscreen = settings.render_settings.fullscreen;

    miniquad::start(config, move |mut ctx| {
        UserData::owning(GameEngine::new(game, settings, &mut ctx), ctx)
    });
}
