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
use hecs::Component;
use serde::Deserialize;

use crate::assets::*;
use crate::audio::*;
use crate::file_loader::FileLoader;
use crate::input::*;
use crate::rendering_engine::RenderingEngine;
use crate::rendering_handler::RenderingHandler;
use crate::resources::Resources;

use self::game_engine::date;
use self::game_engine::GameEngineContext;

pub struct Emerald<'c> {
    delta: f32,
    fps: f64,
    audio_engine: &'c mut Box<dyn AudioEngine>,
    rendering_engine: &'c mut Box<dyn RenderingEngine>,
    input_engine: &'c mut Box<dyn InputEngine>,
    file_loader: &'c mut Box<dyn FileLoader>,
    pub(crate) asset_engine: &'c mut AssetEngine,
    ctx: &'c mut GameEngineContext,
    resources: &'c mut Resources,
}
impl<'c> Emerald<'c> {
    #[inline]
    pub(crate) fn new(
        delta: f32,
        fps: f64,
        audio_engine: &'c mut Box<dyn AudioEngine>,
        input_engine: &'c mut Box<dyn InputEngine>,
        rendering_engine: &'c mut Box<dyn RenderingEngine>,
        file_loader: &'c mut Box<dyn FileLoader>,
        asset_engine: &'c mut AssetEngine,
        ctx: &'c mut GameEngineContext,
        resources: &'c mut Resources,
    ) -> Self {
        Emerald {
            delta,
            fps,
            audio_engine,
            rendering_engine,
            input_engine,
            file_loader,
            asset_engine,
            ctx,
            resources,
        }
    }

    pub fn set_asset_folder_root<T: Into<String>>(&mut self, root: T) {
        self.asset_engine.asset_folder_root = root.into();
    }

    pub fn set_user_data_folder_root(&mut self, root: String) {
        self.asset_engine.user_data_folder_root = root;
    }

    pub fn get_asset_folder_root(&mut self) -> String {
        self.asset_engine.asset_folder_root.clone()
    }

    pub fn get_user_data_folder_root(&mut self) -> String {
        self.asset_engine.user_data_folder_root.clone()
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
    #[deprecated = "Use emd.graphics().screen_size() instead."]
    pub fn screen_size(&self) -> (u32, u32) {
        let size = self.rendering_engine.screen_size();
        (size.width, size.height)
    }

    #[inline]
    pub fn fps(&self) -> f64 {
        self.fps
    }

    /// Requests the game engine to shut down, this will usually happen when the current frame has completed.
    pub fn quit(&mut self) {
        self.ctx.user_requesting_quit = true;
    }
    // *****************************************

    pub fn graphics(&mut self) -> RenderingHandler<'_> {
        RenderingHandler::new(
            &mut self.asset_engine,
            &mut self.rendering_engine,
            &mut self.ctx,
        )
    }

    // ************* Asset API ************* //
    #[inline]
    pub fn loader(&mut self) -> AssetLoader<'_> {
        AssetLoader::new(
            &mut self.asset_engine,
            &mut self.rendering_engine,
            &mut self.audio_engine,
            &mut self.resources,
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
        AudioHandler::new(&mut self.audio_engine, &mut self.asset_engine)
    }
    // ************************************* //

    #[inline]
    pub fn resources(&mut self) -> &mut Resources {
        &mut self.resources
    }

    // ************* Input API ************* //
    #[inline]
    pub fn input(&mut self) -> InputHandler<'_> {
        InputHandler::new(self.input_engine)
    }

    /// Makes all touches also be registered as mouse events.
    #[inline]
    #[deprecated = "Use emd.input().touches_to_mouse(enabled) instead."]
    pub fn touches_to_mouse(&mut self, enabled: bool) {
        // self.input_engine.touches_to_mouse(enabled);
        todo!()
    }

    /// Makes mouse clicks treated as touch event.
    #[inline]
    #[deprecated = "Use emd.input().mouse_to_touch(enabled) instead."]
    pub fn mouse_to_touch(&mut self, enabled: bool) {
        // self.input_engine.mouse_to_touch(enabled);
        todo!()
    }

    #[inline]
    #[deprecated = "Use emd.input().set_key_pressed(keycode, is_pressed) instead."]
    pub fn set_key_pressed(&mut self, keycode: KeyCode, is_pressed: bool) {
        // if is_pressed {
        //     self.input_engine.set_key_down(keycode, false);
        // } else {
        //     self.input_engine.set_key_up(keycode);
        // }
        todo!()
    }
    // ************************************* //
}
