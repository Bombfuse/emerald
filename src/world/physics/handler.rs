use crate::world::*;
use crate::physics::*;
use crate::physics::components::*;
use crate::Vector2;

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

    pub fn create_ground_collider(&mut self, desc: &ColliderDesc<f32>) -> DefaultColliderHandle {
        self.physics_engine.create_ground_collider(&desc)
    }

    pub fn step(&mut self) {
        self.step_n(1);
    }

    pub fn step_n(&mut self, n: u32) {
        self.physics_engine.sync_physics_world_to_game_world(&mut self.world);
        
        for _ in 0..n {
            self.physics_engine.step(&mut self.world);
        }

        self.physics_engine.sync_game_world_to_physics_world(&mut self.world);
    }

    pub fn move_and_collide(&mut self, phb: PhysicsBodyHandle, distance: Vector2<f32>) {
        self.physics_engine.move_and_collide(phb, distance);
        self.physics_engine.sync_game_world_to_physics_world(&mut self.world);
    }

    pub fn move_and_slide(&mut self, phb: PhysicsBodyHandle, distance: Vector2<f32>) {
        self.physics_engine.move_and_slide(phb, distance);

        self.physics_engine.sync_game_entity_position_to_physics_body(&mut self.world, phb);
    }

    pub fn set_gravity(&mut self, gravity: Vector2<f32>) { }

    pub fn set_ccd_substeps(&mut self, substep_count: usize) {
        self.physics_engine
            .mechanical_world
            .integration_parameters
            .max_ccd_substeps = substep_count;
    }

    pub fn set_ccd_max_position_iterations(&mut self, iterations: usize) {
        self.physics_engine
            .mechanical_world
            .integration_parameters
            .max_ccd_position_iterations = iterations;
    }

    pub fn set_ccd_on_penetration_enabled(&mut self, enabled: bool) {
        self.physics_engine
            .mechanical_world
            .integration_parameters
            .ccd_on_penetration_enabled = enabled;
    }
}