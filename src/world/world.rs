use crate::world::physics::*;

use legion::entity::*;
use legion::filter::*;
use legion::world::{TagSet, TagLayout, IntoComponentSource};

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
    
    pub fn insert<T, C>(&mut self, tags: T, components: C) -> &[Entity]
    where
        T: TagSet + TagLayout + for<'a> Filter<ChunksetFilterData<'a>>,
        C: IntoComponentSource,
    { self.inner.insert(tags, components) }

    pub fn physics(&mut self) -> PhysicsHandler {
        PhysicsHandler::new(&mut self.physics_engine, &mut self.inner)
    }
}