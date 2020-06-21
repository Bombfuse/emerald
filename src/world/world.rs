use crate::world::physics::*;

pub struct World {
    pub(crate) physics_engine: PhysicsEngine,
    pub(crate) inner: legion::prelude::World,
}
impl World {
    pub fn new(universe: &mut legion::prelude::Universe) -> Self {
        let physics_engine = PhysicsEngine::new();
        let inner = universe.create_world();

        World {
            physics_engine,
            inner,
        }
    }
}