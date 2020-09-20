use crate::core::*;
use crate::rendering::*;
use crate::input::*;
use crate::world::*;
use crate::logging::*;
use crate::audio::*;
use crate::assets::*;

use instant::Instant;
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
    last_instant: Instant,
    fps_tracker: VecDeque<f64>,
    cache: Cache,
}
impl GameEngine {
    pub fn new(mut game: Box<dyn Game>, settings: GameSettings, mut ctx: &mut Context) -> Self {
        let mut audio_engine = AudioEngine::new();
        let mut input_engine = InputEngine::new();
        let mut rendering_engine = RenderingEngine::new(&mut ctx, settings.render_settings.clone());
        let mut world_engine = WorldEngine::new();
        let mut logging_engine = LoggingEngine::new();

        let base_world = EmeraldWorld::new();
        world_engine.push(base_world);

        let last_instant = Instant::now();
        let now = Instant::now();
        let delta = now - last_instant;

        let starting_amount = 50;
        let mut fps_tracker = VecDeque::with_capacity(starting_amount);
        fps_tracker.resize(starting_amount, 1.0 / 60.0);

        let mut cache = Cache::new();

        let emd = Emerald::new(
            delta,
            0.0,
            &mut ctx,
            &mut audio_engine,
            &mut input_engine,
            &mut world_engine,
            &mut logging_engine,
            &mut rendering_engine,
            &mut cache
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

    /// Return frame rate averaged out over last 600 frames
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
        let start_of_frame = Instant::now();
        let delta = {
            // TODO(bombfuse): Figure out why Instant::now() isn't work on WASM
            // Temporary WASM time hack
            #[cfg(target_arch = "wasm32")]
            {
                std::time::Duration::from_secs_f32(0.016)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                start_of_frame - self.last_instant
            }
        };

        self.last_instant = start_of_frame;

        self.update_fps_tracker(delta.as_secs_f64());
        
        let emd = Emerald::new(
            delta,
            self.get_fps(),
            &mut ctx,
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.world_engine,
            &mut self.logging_engine,
            &mut self.rendering_engine,
            &mut self.cache
        );

        self.game.update(emd);
        self.rendering_engine.update(delta.as_secs_f32(), &mut self.world_engine.world().inner);
        self.audio_engine.frame();
        self.logging_engine.update();
        self.input_engine.rollover();
    }

    #[inline]
    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, repeat: bool) {
        self.input_engine.set_key_down(keycode, repeat);
    }

    #[inline]
    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        self.input_engine.set_key_up(keycode);
    }

    #[inline]
    fn draw(&mut self, mut ctx: &mut Context) {
        let start_of_frame = Instant::now();
        let delta = start_of_frame - self.last_instant;
        
        let emd = Emerald::new(
            delta,
            self.get_fps(),
            &mut ctx,
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.world_engine,
            &mut self.logging_engine,
            &mut self.rendering_engine,
            &mut self.cache
        );

        self.game.draw(emd);
    }
}