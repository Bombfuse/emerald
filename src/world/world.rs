use crate::world::physics::*;

pub struct World {
    physics_engine: PhysicsEngine,
}
impl World {
    pub fn new() -> Self {
        let physics_engine = PhysicsEngine::new();

        World {
            physics_engine,
        }
    }

    pub fn physics_step(&mut self) {}
}