use crate::*;
use crate::physics::PhysicsBodyHandle;

use nphysics2d::object::{
    RigidBodyDesc,
    RigidBody,
    ColliderDesc,
    DefaultColliderHandle,
    BodyPartHandle,
    DefaultBodySet,
    DefaultBodyHandle,
    DefaultColliderSet,
};

use nphysics2d::world::{
    DefaultGeometricalWorld,
    DefaultMechanicalWorld,
};

use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::nalgebra::Isometry2;

pub struct PhysicsEngine {
    pub(crate) geometrical_world: DefaultGeometricalWorld<f32>,
    pub(crate) mechanical_world: DefaultMechanicalWorld<f32>,
    pub(crate) bodies: DefaultBodySet<f32>,
    pub(crate) colliders: DefaultColliderSet<f32>,
    constraints: DefaultJointConstraintSet<f32>,
    forces: DefaultForceGeneratorSet<f32>,
}

impl PhysicsEngine {
    pub(crate) fn new() -> Self {
        let mechanical_world = DefaultMechanicalWorld::new(Vector2::new(0.0, 0.0));
        let geometrical_world = DefaultGeometricalWorld::new();
        let bodies = DefaultBodySet::new();
        let colliders = DefaultColliderSet::new();
        let constraints = DefaultJointConstraintSet::new();
        let forces = DefaultForceGeneratorSet::new();

        PhysicsEngine {
            mechanical_world,
            geometrical_world,
            constraints,
            forces,
            colliders,
            bodies,
        }
    }

    pub(crate) fn step(&mut self, mut world: &mut legion::prelude::World) {
        let non_physics_bodies_query = <(Read<Velocity>, Write<Position>)>::query()
            .filter(!component::<PhysicsBodyHandle>());
        for (vel, mut pos) in non_physics_bodies_query.iter_mut(world) {
            pos.x += vel.linear.x;
            pos.y += vel.linear.y;
        }

        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.constraints,
            &mut self.forces,
        );
    }

    pub(crate) fn create_body(&mut self, desc: &RigidBodyDesc<f32>) -> PhysicsBodyHandle {
        PhysicsBodyHandle::new(self.bodies.insert(desc.build()))
    }

    pub(crate) fn create_collider(&mut self, physics_handle: &mut PhysicsBodyHandle, desc: &ColliderDesc<f32>) -> DefaultColliderHandle {
        let collider = desc.build(
            BodyPartHandle(physics_handle.body_handle, physics_handle.body_part_count as usize)
        );
        let handle = self.colliders.insert(collider);
        physics_handle.add_collider(handle);

        handle
    }

    pub(crate) fn sync_physics_positions_to_positions(&mut self, world: &legion::world::World) {
        let update_positions_query = <(Read<Position>, Read<PhysicsBodyHandle>)>::query();
        let update_velocity_query = <(Read<Velocity>, Read<PhysicsBodyHandle>)>::query();

        for (pos, phb) in update_positions_query.iter(world) {
            self.bodies.rigid_body_mut(phb.body_handle).unwrap()
                .set_position(Isometry2::new(Vector2::new(pos.x, pos.y), std::f32::consts::FRAC_PI_2));
        }

        for (vel, phb) in update_velocity_query.iter(world) {
            self.bodies.rigid_body_mut(phb.body_handle).unwrap()
                .set_velocity(*vel);
        }
    }

    pub(crate) fn sync_positions_to_physics_positions(&mut self, mut world: &mut legion::world::World) {
        let sync_position_query = <(Write<Position>, Read<PhysicsBodyHandle>)>::query();

        for (mut pos, pbh) in sync_position_query.iter_mut(world) {
            let trans = self.bodies.rigid_body_mut(pbh.body_handle).unwrap()
                .position().translation;
            pos.x = trans.x;
            pos.y = trans.y;
        }
    }
}