pub mod assets;
pub mod audio;
pub mod colors;
pub mod core;
pub mod input;
pub mod logging;
pub mod rendering;
pub mod types;
pub mod world;

pub use crate::assets::*;
pub use crate::colors::*;
pub use crate::core::*;
pub use crate::input::*;
pub use crate::rendering::*;
pub use crate::types::*;
pub use crate::world::*;
pub use audio::*;
pub use logging::*;

#[cfg(feature = "physics")]
pub use crate::world::physics::*;

// physics/math/collision re-exports
#[cfg(feature = "physics")]
pub use rapier2d;

#[cfg(feature = "physics")]
pub use rapier2d::{
    crossbeam,
    dynamics::{RigidBody, RigidBodyBuilder, RigidBodyHandle},
    geometry::{Collider, ColliderBuilder, ColliderHandle, InteractionGroups},
    na as nalgebra,
    na::Vector2,
};
//

#[cfg(not(feature = "physics"))]
pub use nalgebra;
#[cfg(not(feature = "physics"))]
pub use nalgebra::Vector2;

// General re-exports for compatibility
pub use hecs;
pub use hecs::*;

pub use nanoserde;
pub use quad_rand as rand;

#[cfg(feature = "gamepads")]
pub use gamepad;
#[cfg(feature = "gamepads")]
pub use gamepad::{Button, Joystick};

use miniquad::{conf, UserData};

pub fn start(game: Box<dyn Game>, settings: GameSettings) {
    let mut config = conf::Conf::default();
    config.window_title = settings.title.clone();
    config.window_width = settings.render_settings.resolution.0 as i32;
    config.window_height = settings.render_settings.resolution.1 as i32;
    config.fullscreen = settings.render_settings.fullscreen;
    config.high_dpi = settings.render_settings.high_dpi;

    miniquad::start(config, move |mut ctx| {
        UserData::owning(GameEngine::new(game, settings, &mut ctx), ctx)
    });
}
