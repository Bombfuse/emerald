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
pub use hecs;
pub use hecs::*;

pub use nanoserde;
pub use quad_rand as rand;

#[cfg(feature = "gamepads")]
pub use gamepad;
#[cfg(feature = "gamepads")]
pub use gamepad::{Button, Joystick};

use miniquad::{conf, UserData};
pub use miniquad::conf::Icon;

pub fn start<G>(game: G, settings: GameSettings)
where
    G: Game + 'static,
{
    let config = conf::Conf {
        window_title: settings.title.clone(),
        window_width: settings.render_settings.resolution.0 as i32,
        window_height: settings.render_settings.resolution.1 as i32,
        fullscreen: settings.render_settings.fullscreen,
        high_dpi: settings.render_settings.high_dpi,
        window_resizable: settings.render_settings.resizable_window,
        icon: settings.render_settings.icon,
        ..Default::default()
    };

    miniquad::start(config, move |mut ctx| {
        UserData::owning(GameEngine::new(game, settings, &mut ctx), ctx)
    });
}
