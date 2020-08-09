use crate::world::*;
use crate::physics::*;
use crate::physics::components::*;
use crate::{Instant, Vector2};

use nphysics2d::object::{RigidBodyDesc, ColliderDesc, DefaultColliderHandle};

pub struct PhysicsHandler<'a> {
    physics_engine: &'a mut  PhysicsEngine,
    world: &'a mut legion::World,
}
impl<'a> PhysicsHandler<'a> {
    pub fn new(physics_engine: &'a mut PhysicsEngine, world: &'a mut legion::World) -> Self {
        PhysicsHandler {
            world,
            physics_engine,
        }
    }

    pub fn create_body(&mut self, desc: &RigidBodyDesc<f32>) -> PhysicsBodyHandle {
        self.physics_engine.create_body(desc)
    }

    pub fn create_collider(&mut self, mut physics_body_handle: &mut PhysicsBodyHandle, desc: &ColliderDesc<f32>) -> DefaultColliderHandle {
        let handle = self.physics_engine.create_collider(&mut physics_body_handle, &desc);
        self.step();

        if let Some(collider) = self.physics_engine.colliders.get(handle.clone()) {
            let bf = self.physics_engine.geometrical_world.broad_phase();
            let aabb = collider
                .proxy_handle()
                .and_then(|h| bf.proxy(h))
                .map(|p| p.0);

            if let Some(aabb) = aabb {
                println!("(x, y): {:?}", (aabb.half_extents().x, aabb.half_extents().y));
            }
        }

        handle
    }

    pub fn create_ground_collider(&mut self, desc: &ColliderDesc<f32>) -> DefaultColliderHandle {
        self.physics_engine.create_ground_collider(&desc)
    }

    pub fn step(&mut self) {
        self.step_n(1);
    }

    pub fn step_n(&mut self, n: u32) {
        let start = Instant::now();

        self.physics_engine.sync_physics_world_to_game_world(&mut self.world);
        
        for _ in 0..n {
            self.physics_engine.step(&mut self.world);
        }

        self.physics_engine.sync_game_world_to_physics_world(&mut self.world);

        let end = Instant::now();
        // println!("Physics Step Duration: {:?}", end - start);
    }

    // pub fn set_gravity(&mut self, gravity: Vector2<f32>) { }

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