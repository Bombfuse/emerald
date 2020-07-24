mod core;
mod rendering;
mod input;
mod world;
mod types;
mod assets;
mod logging;

pub use crate::assets::*;
pub use crate::core::*;
pub use crate::rendering::*;
pub use crate::input::*;
pub use crate::world::*;
pub use crate::types::*;
pub use paintbrush::*;
pub use logging::*;

// physics/math/collision re-exports
pub type Velocity = nphysics2d::math::Velocity<f32>;
pub use nphysics2d::nalgebra::Vector2;
//

// General re-exports for compatibility
pub use instant::Instant;
pub use nanoserde;

use miniquad::{conf, UserData};

pub fn start(game: Box<dyn Game>, settings: GameSettings) {
    let mut config = conf::Conf::default();
    config.window_title = settings.title.clone();
    config.window_width = settings.render_settings.window_size.0 as i32;
    config.window_height = settings.render_settings.window_size.1 as i32;
    config.fullscreen = settings.render_settings.fullscreen;

    miniquad::start(config, move |mut ctx| {
        UserData::owning(GameEngine::new(game, settings, &mut ctx), ctx)
    });
}
