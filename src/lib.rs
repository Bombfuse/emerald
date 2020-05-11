mod core;
mod rendering;
mod input;
mod world;

pub use crate::core::*;
pub use crate::rendering::*;
pub use crate::input::*;
pub use crate::world::*;
pub use paintbrush::*;

use miniquad::*;

pub fn start(game: Box<dyn Game>, settings: GameSettings) {
    miniquad::start(conf::Conf::default(), move |mut ctx| {
        UserData::owning(GameEngine::new(game, settings, &mut ctx), ctx)
    });
}
