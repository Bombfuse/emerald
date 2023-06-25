use std::collections::VecDeque;

use crate::{
    rendering_engine::RenderingEngine, resources::Resources, AssetEngine, AudioEngine, Emerald,
    EmeraldError, Game, GameSettings, InputEngine,
};

pub struct GameEngineContext {
    pub user_requesting_quit: bool,
}
impl GameEngineContext {}

pub struct GameEngine {
    game: Box<dyn Game>,
    rendering_engine: Box<dyn RenderingEngine>,
    audio_engine: Box<dyn AudioEngine>,
    input_engine: Box<dyn InputEngine>,
    resources: Resources,
    last_instant: f64,
    fps_tracker: VecDeque<f64>,

    // Declare last so that it drops last, needed so that asset ref channels stay open while game is dropped
    asset_engine: AssetEngine,
}
impl GameEngine {
    pub fn new(
        game: Box<dyn Game>,
        rendering_engine: Box<dyn RenderingEngine>,
        audio_engine: Box<dyn AudioEngine>,
        input_engine: Box<dyn InputEngine>,
        settings: &GameSettings,
    ) -> Result<Self, EmeraldError> {
        let asset_engine = AssetEngine::new();
        let starting_amount = 50;
        let mut fps_tracker = VecDeque::with_capacity(starting_amount);
        fps_tracker.resize(starting_amount, 1.0 / 60.0);

        Ok(Self {
            game,
            rendering_engine,
            asset_engine,
            audio_engine,
            input_engine,
            last_instant: date::now(),
            fps_tracker,
            resources: Resources::new(),
        })
    }

    pub fn initialize(&mut self, ctx: &mut GameEngineContext) -> Result<(), EmeraldError> {
        let now = date::now();
        let delta = now - self.last_instant;
        self.update_fps_tracker(delta);

        let emd = Emerald::new(
            delta as f32,
            self.get_fps(),
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.rendering_engine,
            &mut self.asset_engine,
            ctx,
            &mut self.resources,
        );

        self.game.initialize(emd);

        Ok(())
    }

    pub fn update(&mut self, ctx: &mut GameEngineContext) -> Result<(), EmeraldError> {
        let now = date::now();
        let delta = now - self.last_instant;
        self.last_instant = now;
        self.update_fps_tracker(delta);

        let emd = Emerald::new(
            delta as f32,
            self.get_fps(),
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.rendering_engine,
            &mut self.asset_engine,
            ctx,
            &mut self.resources,
        );

        self.game.update(emd);
        self.input_engine.update_and_rollover();
        self.audio_engine.post_update().unwrap();
        self.asset_engine.update().unwrap();

        Ok(())
    }

    pub fn render(&mut self, ctx: &mut GameEngineContext) -> Result<(), EmeraldError> {
        let start_of_frame = date::now();
        let delta = start_of_frame - self.last_instant;

        let emd = Emerald::new(
            delta as f32,
            self.get_fps(),
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.rendering_engine,
            &mut self.asset_engine,
            ctx,
            &mut self.resources,
        );
        self.game.draw(emd);

        Ok(())
    }

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

pub(crate) mod date {
    pub fn now() -> f64 {
        instant::now() / 1000.0
    }
}
