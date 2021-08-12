mod components;
mod engine;
mod error;
mod game;
mod game_settings;

pub use components::*;
pub use engine::GameEngine;
pub use error::*;
pub use game::*;
pub use game_settings::*;

use crate::assets::*;
use crate::audio::*;
use crate::input::*;
use crate::logging::*;
use crate::rendering::*;
use crate::world::*;

pub struct Emerald<'a> {
    delta: f32,
    fps: f64,
    audio_engine: &'a mut AudioEngine,
    quad_ctx: &'a mut miniquad::Context,
    rendering_engine: &'a mut RenderingEngine,
    logging_engine: &'a mut LoggingEngine,
    input_engine: &'a mut InputEngine,
    world_engine: &'a mut WorldEngine,
    asset_store: &'a mut AssetStore,
}
impl<'a> Emerald<'a> {
    #[inline]
    pub(crate) fn new(
        delta: f32,
        fps: f64,
        quad_ctx: &'a mut miniquad::Context,
        audio_engine: &'a mut AudioEngine,
        input_engine: &'a mut InputEngine,
        world_engine: &'a mut WorldEngine,
        logging_engine: &'a mut LoggingEngine,
        rendering_engine: &'a mut RenderingEngine,
        asset_store: &'a mut AssetStore,
    ) -> Self {
        Emerald {
            delta,
            fps,
            audio_engine,
            quad_ctx,
            rendering_engine,
            input_engine,
            logging_engine,
            world_engine,
            asset_store,
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

    // ************* General API ***************
    #[inline]
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// Time since Epoch
    #[inline]
    pub fn now(&self) -> f64 {
        miniquad::date::now()
    }

    #[inline]
    pub fn screen_size(&self) -> (f32, f32) {
        self.quad_ctx.screen_size()
    }

    #[inline]
    pub fn fps(&self) -> f64 {
        self.fps
    }

    pub fn quit(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.audio_engine.clear().is_err() {
                //ignore
            }
        }

        self.quad_ctx.quit()
    }
    // *****************************************

    pub fn graphics(&mut self) -> GraphicsHandler<'_> {
        GraphicsHandler::new(
            &mut self.quad_ctx,
            &mut self.asset_store,
            &mut self.rendering_engine,
        )
    }

    // ************* Asset API ************* //
    #[inline]
    pub fn loader(&mut self) -> AssetLoader<'_> {
        AssetLoader::new(
            &mut self.quad_ctx,
            &mut self.asset_store,
            &mut self.rendering_engine,
            &mut self.audio_engine,
        )
    }

    #[inline]
    pub fn writer(&mut self) -> Writer {
        Writer::new()
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
    pub fn input(&mut self) -> InputHandler {
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

    // ************************************* //

    // ************* World API ************* //
    #[inline]
    pub fn world(&mut self) -> &mut EmeraldWorld {
        self.world_engine.world()
    }

    #[inline]
    pub fn world_ref(&self) -> &EmeraldWorld {
        self.world_engine.world_ref()
    }

    #[inline]
    pub fn pop_world(&mut self) -> Option<EmeraldWorld> {
        self.world_engine.pop()
    }

    #[inline]
    pub fn push_world(&mut self, world: EmeraldWorld) {
        self.world_engine.push(world)
    }
    // ************************************* //
}
