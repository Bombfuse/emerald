use std::collections::VecDeque;

use crate::{
    file_loader::FileLoader, rendering_engine::RenderingEngine, resources::Resources,
    schedule::Schedule, system::get_system, world_stack::WorldStack, AssetEngine, AudioEngine,
    Emerald, EmeraldError, Game, GameSettings, InputEngine, World,
};

use super::project::Project;

pub struct GameEngineContext {
    pub user_requesting_quit: bool,
}
impl GameEngineContext {}

pub struct GameEngine {
    pub rendering_engine: Box<dyn RenderingEngine>,
    pub audio_engine: Box<dyn AudioEngine>,
    pub input_engine: Box<dyn InputEngine>,
    pub file_loader: Box<dyn FileLoader>,
    pub schedule: Schedule,
    pub world_stack: WorldStack,
    project: Project,
    resources: Resources,
    last_instant: f64,
    fps_tracker: VecDeque<f64>,

    // Declare last so that it drops last, needed so that asset ref channels stay open while game is dropped
    asset_engine: AssetEngine,
}
impl GameEngine {
    pub fn new(
        project: Project,
        rendering_engine: Box<dyn RenderingEngine>,
        audio_engine: Box<dyn AudioEngine>,
        input_engine: Box<dyn InputEngine>,
        file_loader: Box<dyn FileLoader>,
        asset_engine: AssetEngine,
        settings: &GameSettings,
    ) -> Result<Self, EmeraldError> {
        let starting_amount = 50;
        let mut fps_tracker = VecDeque::with_capacity(starting_amount);
        fps_tracker.resize(starting_amount, 1.0 / 60.0);

        Ok(Self {
            rendering_engine,
            file_loader,
            asset_engine,
            audio_engine,
            input_engine,
            last_instant: date::now(),
            fps_tracker,
            resources: Resources::new(),
            world_stack: WorldStack::new(),
            project,
            schedule: Schedule::new(),
        })
    }

    pub fn initialize(&mut self, ctx: &mut GameEngineContext) -> Result<(), EmeraldError> {
        let now = date::now();
        let delta = now - self.last_instant;
        self.update_fps_tracker(delta);

        let mut emd = Emerald::new(
            delta as f32,
            self.get_fps(),
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.rendering_engine,
            &mut self.file_loader,
            &mut self.asset_engine,
            ctx,
            &mut self.resources,
        );
        let world = emd
            .loader()
            .world(&self.project.init_world)
            .unwrap_or(World::new());
        self.schedule = self
            .project
            .schedules
            .get(&self.project.init_schedule)
            .unwrap()
            .clone();
        self.world_stack.push_front(world);

        Ok(())
    }

    pub fn update(&mut self, ctx: &mut GameEngineContext) -> Result<(), EmeraldError> {
        let now = date::now();
        let delta = now - self.last_instant;
        self.last_instant = now;
        self.update_fps_tracker(delta);

        let mut emd = Emerald::new(
            delta as f32,
            self.get_fps(),
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.rendering_engine,
            &mut self.file_loader,
            &mut self.asset_engine,
            ctx,
            &mut self.resources,
        );
        self.schedule.run(&mut emd, &mut self.world_stack);

        if self.world_stack.is_empty() {
            emd.quit();
            return Ok(());
        }

        self.input_engine.update_and_rollover();
        self.audio_engine.post_update().unwrap();
        self.asset_engine.update().unwrap();

        Ok(())
    }

    pub fn render(&mut self, ctx: &mut GameEngineContext) -> Result<(), EmeraldError> {
        let start_of_frame = date::now();
        let delta = start_of_frame - self.last_instant;

        let mut emd = Emerald::new(
            delta as f32,
            self.get_fps(),
            &mut self.audio_engine,
            &mut self.input_engine,
            &mut self.rendering_engine,
            &mut self.file_loader,
            &mut self.asset_engine,
            ctx,
            &mut self.resources,
        );
        let render_system = get_system(&mut emd, "render_system").unwrap_or(default_render_system);
        (render_system)(&mut emd, self.world_stack.front_mut().unwrap());

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

fn default_render_system(emd: &mut Emerald, world: &mut World) {
    emd.graphics().begin().unwrap();
    emd.graphics().draw_world(world).unwrap();
    emd.graphics().render().unwrap();
}

pub(crate) mod date {
    pub fn now() -> f64 {
        instant::now() / 1000.0
    }
}
