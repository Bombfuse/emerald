use crate::core::*;
use crate::rendering::*;
use crate::input::*;

use miniquad::*;
use std::time::{Instant, Duration};

pub struct GameEngine {
    game: Box<dyn Game>,
    settings: GameSettings,
    input_engine: InputEngine,
    rendering_engine: RenderingEngine,
}
impl GameEngine {
    pub fn new(mut game: Box<dyn Game>, settings: GameSettings, mut ctx: &mut Context) -> Self {
        let input_engine = InputEngine::new();
        let rendering_engine = RenderingEngine::new();

        GameEngine {
            game,
            settings,
            input_engine,
            rendering_engine,
        }
    }
}
impl EventHandler for GameEngine {
    fn update(&mut self, _ctx: &mut Context) {

    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        println!("{:?}", (x, y));
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        self.input_engine.key_down(keycode);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        self.input_engine.key_up(keycode);
    }



    fn draw(&mut self, mut ctx: &mut Context) {
        self.rendering_engine.update(&mut ctx);
    }
}