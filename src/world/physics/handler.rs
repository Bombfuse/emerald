use crate::world::*;
use crate::physics::*;


pub struct PhysicsHandler<'a> {
    physics_engine: &'a mut  PhysicsEngine,
    world: &'a mut legion::prelude::World,
}
impl<'a> PhysicsHandler<'a> {
    pub fn new(physics_engine: &'a mut PhysicsEngine, world: &'a mut legion::prelude::World) -> Self {
        PhysicsHandler {
            world,
            physics_engine,
        }
    }

    pub fn step(&mut self) {
        self.step_n(1);
    }

    pub fn step_n(&mut self, n: u32) {
        for i in 0..n {
            self.physics_engine.step(&mut self.world)
        }
    }
}