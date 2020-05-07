mod core;
mod rendering;
mod input;

pub use crate::core::*;
pub use crate::rendering::*;
pub use crate::input::*;

use miniquad::*;

pub fn start(mut game: Box<dyn Game>, settings: GameSettings) {
    miniquad::start(conf::Conf::default(), move |mut ctx| {
        UserData::owning(GameEngine::new(game, settings, &mut ctx), ctx)
    });
}
