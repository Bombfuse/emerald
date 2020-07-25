use crate::world::*;
use crate::physics::*;
use crate::physics::components::*;

use nphysics2d::object::{RigidBodyDesc, ColliderDesc, DefaultColliderHandle};

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

    pub fn create_body(&mut self, desc: &RigidBodyDesc<f32>) -> PhysicsBodyHandle {
        self.physics_engine.create_body(desc)
    }

    pub fn create_collider(&mut self, mut physics_body_handle: &mut PhysicsBodyHandle, desc: &ColliderDesc<f32>) -> DefaultColliderHandle {
        self.physics_engine.create_collider(&mut physics_body_handle, &desc)
    }

    pub fn step(&mut self) {
        self.step_n(1);
    }

    pub fn step_n(&mut self, n: u32) {
        for _ in 0..n {
            self.physics_engine.step(&mut self.world)
        }
    }
}