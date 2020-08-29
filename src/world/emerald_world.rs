use crate::world::physics::*;
use hecs::World;

pub struct EmeraldWorld {
    pub(crate) physics_engine: PhysicsEngine,
    pub(crate) inner: World,
}
impl EmeraldWorld {
    pub fn new() -> Self {
        EmeraldWorld {
            physics_engine: PhysicsEngine::new(),
            inner: World::default(),
        }
    }
}