use crate::core::*;
use crate::rendering::*;
use crate::input::*;
use crate::world::*;
use crate::logging::*;

use instant::Instant;
use miniquad::*;

pub struct GameEngine {
    game: Box<dyn Game>,
    _settings: GameSettings,
    input_engine: InputEngine,
    logging_engine: LoggingEngine,
    rendering_engine: RenderingEngine,
    world_engine: WorldEngine,
    last_instant: Instant,
}
impl GameEngine {
    pub fn new(mut game: Box<dyn Game>, settings: GameSettings, mut ctx: &mut Context) -> Self {
        let mut input_engine = InputEngine::new();
        let mut rendering_engine = RenderingEngine::new(&mut ctx, settings.render_settings.clone());
        let mut world_engine = WorldEngine::new();
        let mut logging_engine = LoggingEngine::new();

        let base_world = world_engine.create();
        world_engine.push(base_world);

        let last_instant = Instant::now();
        let now = Instant::now();
        let delta = now - last_instant;

        let emd = Emerald::new(
            delta,
            &mut ctx,
            &mut input_engine,
            &mut world_engine,
            &mut logging_engine,
            &mut rendering_engine);

        game.initialize(emd);

        GameEngine {
            game,
            _settings: settings,
            input_engine,
            logging_engine,
            rendering_engine,
            world_engine,
            last_instant,
        }
    }
}
impl EventHandler for GameEngine {
    fn update(&mut self, mut ctx: &mut Context) {
        let now = Instant::now();
        let delta = now - self.last_instant;

        
        let emd = Emerald::new(
            delta,
            &mut ctx,
            &mut self.input_engine,
            &mut self.world_engine,
            &mut self.logging_engine,
            &mut self.rendering_engine);

        self.game.update(emd);
        self.logging_engine.update();
        self.last_instant = now;

        self.input_engine.rollover();
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) {}

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, repeat: bool) {
        self.input_engine.set_key_down(keycode, repeat);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        self.input_engine.set_key_up(keycode);
    }

    fn draw(&mut self, mut ctx: &mut Context) {
        self.rendering_engine.update(&mut ctx, self.world_engine.world());
    }
}