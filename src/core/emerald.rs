use crate::assets::*;
use crate::audio::*;
use crate::input::*;
use crate::logging::*;
use crate::rendering::*;
use crate::world::*;
use crate::EmeraldError;

use hecs::Entity;
use std::time::Instant;

pub struct Emerald<'a> {
    delta: f32,
    fps: f64,
    audio_engine: &'a mut AudioEngine,
    quad_ctx: &'a mut miniquad::Context,
    rendering_engine: &'a mut RenderingEngine,
    logging_engine: &'a mut LoggingEngine,
    input_engine: &'a mut InputEngine,
    world_engine: &'a mut WorldEngine,
    cache: &'a mut Cache,
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
        cache: &'a mut Cache,
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
            cache,
        }
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
        self.audio_engine.clear();
        self.quad_ctx.quit()
    }
    // *****************************************

    /// Disable all cameras then set the camera on the given entity as active.
    /// Fails if the given entity does not exist, or does not have a camera.
    #[inline]
    pub fn make_active_camera(&mut self, entity: Entity) -> Result<(), EmeraldError> {
        self.rendering_engine
            .make_active_camera(entity, self.world_engine.world())
    }

    pub fn graphics(&mut self) -> GraphicsHandler {
        GraphicsHandler::new(
            &mut self.quad_ctx,
            &mut self.rendering_engine,
            self.world_engine,
        )
    }

    // ************* Asset API ************* //
    #[inline]
    pub fn loader(&mut self) -> AssetLoader {
        AssetLoader::new(
            &mut self.quad_ctx,
            &mut self.rendering_engine,
            &mut self.audio_engine,
            &mut self.cache,
        )
    }
    // ************************************* //

    // ************* Audio API ************* //
    #[inline]
    pub fn audio(&mut self) -> AudioHandler {
        AudioHandler::new(&mut self.audio_engine)
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
        InputHandler::new(&mut self.input_engine)
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
