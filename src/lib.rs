mod core;
mod rendering;
mod input;
mod world;
mod types;
mod assets;

pub use crate::assets::*;
pub use crate::core::*;
pub use crate::rendering::*;
pub use crate::input::*;
pub use crate::world::*;
pub use crate::types::*;
pub use paintbrush::*;

// physics/math/collision re-exports
pub use nphysics2d::nalgebra::Vector2;
//

use miniquad::{conf, UserData};

pub fn start(game: Box<dyn Game>, settings: GameSettings) {
    miniquad::start(conf::Conf::default(), move |mut ctx| {
        UserData::owning(GameEngine::new(game, settings, &mut ctx), ctx)
    });
}
