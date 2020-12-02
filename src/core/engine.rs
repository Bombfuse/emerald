use crate::assets::*;
use crate::audio::*;
use crate::core::*;
use crate::input::*;
use crate::logging::*;
use crate::rendering::*;
use crate::world::*;

use miniquad::*;
use std::collections::VecDeque;

pub struct GameEngine {
    game: Box<dyn Game>,
    _settings: GameSettings,
    audio_engine: AudioEngine,
    input_engine: InputEngine,
    logging_engine: LoggingEngine,
    rendering_engine: RenderingEngine,
    world_engine: WorldEngine,
    last_instant: f64,
    fps_tracker: VecDeque<f64>,
    cache: Cache,
}
impl GameEngine {
    pub fn new(mut game: Box<dyn Game>, settings: GameSettings, mut ctx: &mut Context) -> Self {
        let mut logging_engine = LoggingEngine::new();
        let mut audio_engine = AudioEngine::new();
        let mut input_engine = InputEngine::new();
        let mut rendering_engine = RenderingEngine::new(&mut ctx, settings.render_settings.clone());
        let mut world_engine = WorldEngine::new();

        world_engine.push(EmeraldWorld::new());

        let delta = 0.0;
        let starting_amount = 50;
        let mut fps_tracker = VecDeque::with_capacity(starting_amount);
        fps_tracker.resize(starting_amount, 1.0 / 60.0);
        let mut cache = Cache::new();
        let last_instant = miniquad::date::now();

        let emd = Emerald::new(
            delta,
            0.0,
            &mut ctx,
            &mut audio_engine,
            &mut input_engine,
            &mut world_engine,
            &mut logging_engine,
            &mut rendering_engine,
            &mut cache,
        );

        game.initialize(emd);

        GameEngine {
            game,
            fps_tracker,
            _settings: settings,
            audio_engine,
            input_engine,
            logging_engine,
            rendering_engine,
            world_engine,
            last_instant,
            cache,
        }
    }

    /// Return frame rate averaged out over last N frames
    /// https://github.com/17cupsofcoffee/tetra/blob/master/src/time.rs
    #[inline]
    pub fn get_fps(&self) -> f64 {
        1.0 / (self.fps_tracker.iter().sum::<f64>() / self.fps_tracker.len() as f64)
    }

    #[inline]
    fn update_fps_tracker(&mut self, delta: f64) {
        self.fps_tracker.pop_front();
        self.fps_tracker.push_back(delta);
    }
}
impl EventHandler for GameEngine {
    #[inline]
    fn update(&mut self, mut ctx: &mut Context) {
        let start_of_frame = miniquad::date::now();
        let delta = start_of_frame - self.last_instant;
        self.last_instant = start_of_frame;

        self.update_fps_tracker(delta);

        let emd = Emerald::new(
            delta as f32,
            self.get_fps(),
            &mut ctx,
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.world_engine,
            &mut self.logging_engine,
            &mut self.rendering_engine,
            &mut self.cache,
        );

        self.game.update(emd);
        self.rendering_engine
            .update(delta, &mut self.world_engine.world().inner);
        self.audio_engine.frame();
        self.logging_engine.update().unwrap();
        self.input_engine.update_and_rollover().unwrap();
    }

    #[inline]
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        repeat: bool,
    ) {
        self.input_engine.set_key_down(keycode, repeat);
    }

    #[inline]
    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        self.input_engine.set_key_up(keycode);
    }

    #[inline]
    fn draw(&mut self, mut ctx: &mut Context) {
        let start_of_frame = miniquad::date::now();
        let delta = start_of_frame - self.last_instant;

        let emd = Emerald::new(
            delta as f32,
            self.get_fps(),
            &mut ctx,
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.world_engine,
            &mut self.logging_engine,
            &mut self.rendering_engine,
            &mut self.cache,
        );

        self.game.draw(emd);
    }
}
