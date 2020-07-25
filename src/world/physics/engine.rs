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

pub struct PhysicsEngine {
    geometrical_world: DefaultGeometricalWorld<f32>,
    mechanical_world: DefaultMechanicalWorld<f32>,
    bodies: DefaultBodySet<f32>,
    colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,
}

impl PhysicsEngine {
    pub(crate) fn new() -> Self {
        let mechanical_world = DefaultMechanicalWorld::new(Vector2::new(0.0, -9.81));
        let geometrical_world = DefaultGeometricalWorld::new();
        let bodies = DefaultBodySet::new();
        let colliders = DefaultColliderSet::new();
        let joint_constraints = DefaultJointConstraintSet::new();
        let force_generators = DefaultForceGeneratorSet::new();

        PhysicsEngine {
            mechanical_world,
            geometrical_world,
            joint_constraints,
            force_generators,
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
}