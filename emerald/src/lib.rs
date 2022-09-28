#![deny(
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]

pub mod assets;
pub mod audio;
pub mod colors;
pub mod core;
pub mod input;
pub mod logging;
pub mod profiling;
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

pub use glam;

// physics/math/collision re-exports
#[cfg(feature = "physics")]
pub use rapier2d;

#[cfg(feature = "physics")]
pub use rapier2d::{
    crossbeam,
    dynamics::{RigidBody, RigidBodyBuilder, RigidBodyHandle},
    geometry::{Collider, ColliderBuilder, ColliderHandle, InteractionGroups, Ray},
    na as nalgebra,
    na::Vector2,
    parry,
};
//

#[cfg(not(feature = "physics"))]
pub use nalgebra;
#[cfg(not(feature = "physics"))]
pub use nalgebra::Vector2;

// General re-exports for compatibility
pub use hecs::Entity;

pub use nanoserde;
pub use quad_rand as rand;

#[cfg(feature = "gamepads")]
pub use gamepad;
#[cfg(feature = "gamepads")]
pub use gamepad::{Button, Joystick};

use miniquad::conf;

pub fn start(game: Box<dyn Game>, settings: GameSettings) {
    let mut config = conf::Conf::default();
    config.window_title = settings.title.clone();
    config.window_width = settings.render_settings.resolution.0 as i32;
    config.window_height = settings.render_settings.resolution.1 as i32;
    config.fullscreen = settings.render_settings.fullscreen;
    config.high_dpi = settings.render_settings.high_dpi;
    config.window_resizable = settings.render_settings.resizable_window;
    config.icon = settings.render_settings.icon.clone();

    miniquad::start(config, move |mut ctx| {
        Box::new(GameEngine::new(game, settings, &mut ctx))
    });
}
