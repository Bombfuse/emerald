pub mod components;
pub mod error;
pub mod game;
pub mod game_engine;
pub mod game_settings;

pub use components::transform::*;
pub use components::*;
pub use error::*;
pub use game::*;
pub use game_settings::*;
use winit::window::CursorIcon;

use crate::assets::*;
use crate::audio::*;
use crate::input::*;
use crate::logging::*;
use crate::profiling::profile_cache::ProfileCache;
use crate::profiling::profiler::Profiler;
use crate::rendering_engine::RenderingEngine;
use crate::rendering_handler::RenderingHandler;

use self::game_engine::date;
use self::game_engine::GameEngineContext;

pub struct Emerald<'c> {
    delta: f32,
    fps: f64,
    audio_engine: &'c mut AudioEngine,
    rendering_engine: &'c mut RenderingEngine,
    logging_engine: &'c mut LoggingEngine,
    input_engine: &'c mut InputEngine,
    pub(crate) asset_store: &'c mut AssetStore,
    profile_cache: &'c mut ProfileCache,
    ctx: &'c mut GameEngineContext,
}
impl<'c> Emerald<'c> {
    #[inline]
    pub(crate) fn new(
        delta: f32,
        fps: f64,
        audio_engine: &'c mut AudioEngine,
        input_engine: &'c mut InputEngine,
        logging_engine: &'c mut LoggingEngine,
        rendering_engine: &'c mut RenderingEngine,
        asset_store: &'c mut AssetStore,
        profile_cache: &'c mut ProfileCache,
        ctx: &'c mut GameEngineContext,
    ) -> Self {
        Emerald {
            delta,
            fps,
            audio_engine,
            rendering_engine,
            input_engine,
            logging_engine,
            asset_store,
            profile_cache,
            ctx,
        }
    }

    pub fn set_asset_folder_root(&mut self, root: String) {
        self.asset_store.set_asset_folder_root(root);
    }

    pub fn set_user_data_folder_root(&mut self, root: String) {
        self.asset_store.set_user_data_folder_root(root);
    }

    pub fn get_asset_folder_root(&mut self) -> String {
        self.asset_store.get_asset_folder_root()
    }

    pub fn get_user_data_folder_root(&mut self) -> String {
        self.asset_store.get_user_data_folder_root()
    }

    pub fn set_cursor(&mut self, cursor: CursorIcon) {
        if let Some(window) = &self.ctx.window {
            window.set_cursor_icon(cursor);
        }
    }

    // ************* General API ***************
    #[inline]
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// WARNING: This overrides the delta value set by the emerald engine.
    #[inline]
    pub fn set_delta(&mut self, delta: f32) {
        self.delta = delta;
    }

    /// Time since Epoch
    #[inline]
    pub fn now(&self) -> f64 {
        date::now()
    }

    #[inline]
    pub fn screen_size(&self) -> (u32, u32) {
        (
            self.rendering_engine.size.width,
            self.rendering_engine.size.height,
        )
    }

    #[inline]
    pub fn fps(&self) -> f64 {
        self.fps
    }

    pub fn quit(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.audio_engine.clear().ok();
        }

        todo!()
    }
    // *****************************************

    pub fn graphics(&mut self) -> RenderingHandler<'_> {
        RenderingHandler::new(
            &mut self.asset_store,
            &mut self.rendering_engine,
            &mut self.ctx,
        )
    }

    pub fn profiler<T: Into<String>>(&mut self, profile_name: T) -> Profiler<'_> {
        let now = self.now();

        Profiler::new(&mut self.profile_cache, profile_name, now)
    }

    // ************* Asset API ************* //
    #[inline]
    pub fn loader(&mut self) -> AssetLoader<'_> {
        AssetLoader::new(
            &mut self.asset_store,
            &mut self.rendering_engine,
            &mut self.audio_engine,
        )
    }

    #[inline]
    pub fn writer(&mut self) -> Writer {
        Writer::new(self.get_user_data_folder_root())
    }

    // ************************************* //

    // ************* Audio API ************* //
    #[inline]
    pub fn audio(&mut self) -> AudioHandler<'_> {
        AudioHandler::new(&mut self.audio_engine, &mut self.asset_store)
    }
    // ************************************* //

    /// Logging
    // ************* Logging API ************* //
    #[inline]
    pub fn logger(&mut self) -> &mut LoggingEngine {
        &mut self.logging_engine
    }
    // ************************************* //

    // ************* Input API ************* //
    #[inline]
    pub fn input(&mut self) -> InputHandler<'_> {
        InputHandler::new(self.input_engine)
    }

    /// Makes all touches also be registered as mouse events.
    #[inline]
    pub fn touches_to_mouse(&mut self, enabled: bool) {
        self.input_engine.touches_to_mouse = enabled;
    }

    /// Makes mouse clicks treated as touch event.
    #[inline]
    pub fn mouse_to_touch(&mut self, enabled: bool) {
        self.input_engine.mouse_to_touch = enabled;
    }

    #[inline]
    pub fn set_key_pressed(&mut self, keycode: KeyCode, is_pressed: bool) {
        if is_pressed {
            self.input_engine.set_key_down(keycode, false);
        } else {
            self.input_engine.set_key_up(keycode);
        }
    }
    // ************************************* //
}
