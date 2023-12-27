use std::collections::VecDeque;

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, VirtualKeyCode, WindowEvent},
};

use crate::{
    profiling::profile_cache::ProfileCache, rendering_engine::RenderingEngine,
    resources::Resources, AssetEngine, AudioEngine, Emerald, EmeraldError, Game, GameSettings,
    InputEngine, LoggingEngine,
};

pub(crate) struct GameEngineContext {
    pub window: Option<winit::window::Window>,
    pub user_requesting_quit: bool,
}
impl GameEngineContext {
    pub fn get_window_id(&self) -> Option<winit::window::WindowId> {
        self.window.as_ref().map(|window| window.id().clone())
    }
}

pub(crate) struct GameEngine {
    game: Box<dyn Game>,
    rendering_engine: RenderingEngine,
    audio_engine: AudioEngine,
    profile_cache: ProfileCache,
    input_engine: InputEngine,
    logging_engine: LoggingEngine,
    resources: Resources,
    last_instant: f64,
    fps_tracker: VecDeque<f64>,

    // Declare last so that it drops last, needed so that asset ref channels stay open while game is dropped
    asset_engine: AssetEngine,
}
impl GameEngine {
    pub async fn new(
        game: Box<dyn Game>,
        window: &winit::window::Window,
        settings: &GameSettings,
    ) -> Result<Self, EmeraldError> {
        let mut asset_engine = AssetEngine::new();
        let rendering_engine =
            RenderingEngine::new(window, settings.render_settings.clone(), &mut asset_engine)
                .await?;

        let audio_engine = AudioEngine::new();
        let input_engine = InputEngine::new();
        let profile_cache = ProfileCache::new(Default::default());
        let logging_engine = LoggingEngine::new();

        let starting_amount = 100;
        let mut fps_tracker = VecDeque::with_capacity(starting_amount);
        fps_tracker.resize(starting_amount, 1.0 / 60.0);

        Ok(Self {
            game,
            rendering_engine,
            logging_engine,
            asset_engine,
            audio_engine,
            input_engine,
            profile_cache,
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
            &mut self.logging_engine,
            &mut self.rendering_engine,
            &mut self.asset_engine,
            &mut self.profile_cache,
            ctx,
            &mut self.resources,
        );

        self.game.initialize(emd);

        Ok(())
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn resize_window(&mut self, new_size: PhysicalSize<u32>) {
        self.rendering_engine.resize_window(new_size);
    }

    pub fn window_size(&self) -> PhysicalSize<u32> {
        self.rendering_engine.size
    }

    pub fn handle_cursor_move(&mut self, position: &winit::dpi::PhysicalPosition<f64>) {
        self.input_engine.handle_cursor_move(position)
    }

    pub fn handle_mouse_input(&mut self, button: &winit::event::MouseButton, state: &ElementState) {
        self.input_engine.handle_mouse_input(button, state)
    }

    pub fn handle_virtual_keycode(&mut self, virtual_keycode: VirtualKeyCode, state: ElementState) {
        self.input_engine
            .handle_virtual_keycode(virtual_keycode, state)
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
            &mut self.logging_engine,
            &mut self.rendering_engine,
            &mut self.asset_engine,
            &mut self.profile_cache,
            ctx,
            &mut self.resources,
        );

        self.game.update(emd);
        self.logging_engine.update().unwrap();
        self.input_engine.update_and_rollover().unwrap();
        self.audio_engine.post_update().unwrap();
        self.asset_engine.update().unwrap();

        Ok(())
    }

    pub fn render(&mut self, ctx: &mut GameEngineContext) -> Result<(), wgpu::SurfaceError> {
        let start_of_frame = date::now();
        let delta = start_of_frame - self.last_instant;

        let emd = Emerald::new(
            delta as f32,
            self.get_fps(),
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.logging_engine,
            &mut self.rendering_engine,
            &mut self.asset_engine,
            &mut self.profile_cache,
            ctx,
            &mut self.resources,
        );
        self.game.draw(emd);

        Ok(())
    }

    /// Final task before exiting program
    pub fn clean_up(mut self) {
        drop(self.game);
        drop(self.rendering_engine);
        drop(self.audio_engine);
        drop(self.resources);

        // Clean up all assets
        self.asset_engine.update().unwrap();
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
