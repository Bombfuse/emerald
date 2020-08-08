mod core;
mod rendering;
mod input;
mod world;
mod types;
mod assets;
mod logging;
mod audio;

pub use crate::assets::*;
pub use crate::core::*;
pub use crate::rendering::*;
pub use crate::input::*;
pub use crate::world::*;
pub use crate::world::physics::*;
pub use crate::types::*;
pub use paintbrush::*;
pub use logging::*;
pub use audio::*;

// physics/math/collision re-exports
pub type Velocity = nphysics2d::math::Velocity<f32>;
pub use nphysics2d::nalgebra::Vector2;
pub use nphysics2d::object::{
    RigidBody,
    RigidBodyDesc,
    ColliderDesc,
    Collider,
    BodyStatus,
};
pub use nphysics2d::ncollide2d::shape::{ShapeHandle, Ball, Cuboid};
//

// General re-exports for compatibility
pub use instant::Instant;
pub use nanoserde;
pub use nphysics2d;

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
