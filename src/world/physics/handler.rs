use crate::world::*;
use crate::physics::*;
use crate::physics::components::*;
use crate::{Instant, Vector2};

use rapier2d::geometry::{ColliderHandle, ColliderBuilder, Collider};
use rapier2d::dynamics::{RigidBodyHandle, RigidBodyBuilder, RigidBody, RigidBodyMut};

pub struct PhysicsHandler<'a> {
    physics_engine: &'a mut  PhysicsEngine,
    world: &'a mut hecs::World,
}
impl<'a> PhysicsHandler<'a> {
    pub fn new(physics_engine: &'a mut PhysicsEngine, world: &'a mut hecs::World) -> Self {
        PhysicsHandler {
            world,
            physics_engine,
        }
    }

    pub fn create_body(&mut self, desc: &RigidBodyBuilder) -> RigidBodyHandle {
        self.physics_engine.create_body(desc)
    }

    pub fn create_collider(&mut self, body_handle: RigidBodyHandle, desc: &ColliderBuilder) -> ColliderHandle {
        let handle = self.physics_engine.create_collider(body_handle, &desc);
        // self.step();

        handle
    }

    pub fn rigid_body_mut(&mut self, body_handle: RigidBodyHandle) -> Option<RigidBodyMut> {
        self.physics_engine.bodies.get_mut(body_handle)
    }

    // pub fn create_ground_collider(&mut self, desc: &ColliderDesc<f32>) -> DefaultColliderHandle {
    //     self.physics_engine.create_ground_collider(&desc)
    // }

    pub fn step(&mut self) {
        self.step_n(1);
    }

    pub fn step_n(&mut self, n: u32) {
        let start = Instant::now();

        self.physics_engine.sync_physics_world_to_game_world(&mut self.world);
        
        for _ in 0..n {
            self.physics_engine.step();
        }

        self.physics_engine.sync_game_world_to_physics_world(&mut self.world);

        let end = Instant::now();
        // println!("Physics Step Duration: {:?}", end - start);
    }

    // pub fn set_gravity(&mut self, gravity: Vector2<f32>) { }

    // pub fn set_ccd_substeps(&mut self, substep_count: usize) {
    //     self.physics_engine
    //         .mechanical_world
    //         .integration_parameters
    //         .max_ccd_substeps = substep_count;
    // }

    // pub fn set_ccd_max_position_iterations(&mut self, iterations: usize) {
    //     self.physics_engine
    //         .mechanical_world
    //         .integration_parameters
    //         .max_ccd_position_iterations = iterations;
    // }

    // pub fn set_ccd_on_penetration_enabled(&mut self, enabled: bool) {
    //     self.physics_engine
    //         .mechanical_world
    //         .integration_parameters
    //         .ccd_on_penetration_enabled = enabled;
    // }
}