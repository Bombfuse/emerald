use crate::world::*;
use crate::world::physics::*;

// This is an API for the WorldEngine, but WorldHandler sounds better than "WorldEngineHandler".
pub struct WorldHandler<'a> {
    engine: &'a mut WorldEngine,
}
impl<'a> WorldHandler<'a> {
    pub fn new(engine: &'a mut WorldEngine) -> Self {
        WorldHandler {
            engine,
        }
    }

    pub fn physics(&mut self) -> PhysicsHandler {
        let world = self.engine.world_mut();

        PhysicsHandler::new(world)
    }
}