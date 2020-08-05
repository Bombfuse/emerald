use crate::*;
use crate::physics::{PhysicsBodyHandle, PhysicsBody};

use nphysics2d::object::{
    RigidBodyDesc,
    RigidBody,
    ColliderDesc,
    DefaultColliderHandle,
    BodyPartHandle,
    DefaultBodySet,
    DefaultBodyHandle,
    DefaultColliderSet,
    Ground,
};

use nphysics2d::world::{
    DefaultGeometricalWorld,
    DefaultMechanicalWorld,
};

use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::nalgebra::{Isometry2, Translation2};


use uuid::Uuid;

use std::collections::HashMap;

/// A physics engine unique to a game world. This handles the RigidBodies of the game.
pub struct PhysicsEngine {
    pub(crate) geometrical_world: DefaultGeometricalWorld<f32>,
    pub(crate) mechanical_world: DefaultMechanicalWorld<f32>,
    pub(crate) bodies: DefaultBodySet<f32>,
    pub(crate) colliders: DefaultColliderSet<f32>,
    pub(crate) physics_bodies: HashMap<PhysicsBodyHandle, PhysicsBody>,
    constraints: DefaultJointConstraintSet<f32>,
    forces: DefaultForceGeneratorSet<f32>,
    ground_handle: Option<DefaultBodyHandle>,
}

impl PhysicsEngine {
    pub(crate) fn new() -> Self {
        let mut mechanical_world = DefaultMechanicalWorld::new(Vector2::new(0.0, 0.0));
        let geometrical_world = DefaultGeometricalWorld::new();
        let bodies = DefaultBodySet::new();
        let colliders = DefaultColliderSet::new();
        let constraints = DefaultJointConstraintSet::new();
        let forces = DefaultForceGeneratorSet::new();
        let physics_bodies = HashMap::new();

        PhysicsEngine {
            mechanical_world,
            geometrical_world,
            constraints,
            forces,
            colliders,
            bodies,
            physics_bodies,
            ground_handle: None,
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
        let body_handle = self.bodies.insert(desc.build());
        let uuid = Uuid::new_v4();
        let physics_body = PhysicsBody::new(body_handle);
        let handle = PhysicsBodyHandle::new(uuid);
        self.physics_bodies.insert(handle, physics_body);

        handle
    }

    pub(crate) fn create_collider(&mut self, physics_handle: &mut PhysicsBodyHandle, desc: &ColliderDesc<f32>) -> DefaultColliderHandle {
        let mut physics_body = self.physics_bodies.get_mut(physics_handle).unwrap();

        let collider = desc.build(
            BodyPartHandle(physics_body.body_handle, physics_body.body_part_count as usize)
        );
        let handle = self.colliders.insert(collider);
        physics_body.add_collider(handle);

        handle
    }

    pub(crate) fn create_ground_collider(&mut self, desc: &ColliderDesc<f32>) -> DefaultColliderHandle {
        if self.ground_handle.is_none() {
            self.ground_handle = Some(self.bodies.insert(Ground::new()));
        }

        let ground_handle = self.ground_handle.unwrap();

        self.colliders.insert(desc.build(BodyPartHandle(ground_handle, 0)))
    }

    pub(crate) fn move_and_collide(&mut self, phb: PhysicsBodyHandle, translation: Vector2<f32>) {}

    pub(crate) fn move_and_slide(&mut self, phb: PhysicsBodyHandle, translation: Vector2<f32>) {
        let physics_body = self.physics_bodies.get_mut(&phb).unwrap();
        let pos = self.bodies.rigid_body_mut(physics_body.body_handle).unwrap().position().clone();

        self.bodies.rigid_body_mut(physics_body.body_handle).unwrap()
            .set_position(Isometry2::new(
                Vector2::new(
                    pos.translation.x + translation.x,
                    pos.translation.y + translation.y
                ),
                std::f32::consts::FRAC_PI_2
            ))
    }

    pub(crate) fn sync_physics_world_to_game_world(&mut self, world: &legion::world::World) {
        let update_positions_query = <(Read<Position>, Read<PhysicsBodyHandle>)>::query();
        let update_velocity_query = <(Read<Velocity>, Read<PhysicsBodyHandle>)>::query();

        for (pos, phb) in update_positions_query.iter(world) {
            self.sync_physics_position_to_entity_position(&pos, *phb);
        }

        for (vel, phb) in update_velocity_query.iter(world) {
            let physics_body = self.physics_bodies.get(&phb).unwrap();
            self.bodies.rigid_body_mut(physics_body.body_handle).unwrap()
                .set_velocity(*vel);
        }
    }

    pub(crate) fn sync_game_world_to_physics_world(&mut self, mut world: &mut legion::world::World) {
        let sync_position_query = <(Write<Position>, Read<PhysicsBodyHandle>)>::query();
        let sync_velocity_query = <(Write<Velocity>, Read<PhysicsBodyHandle>)>::query();

        for (mut pos, phb) in sync_position_query.iter_mut(world) {
            self.sync_entity_position_to_physics_position(pos, *phb);
        }

        for (mut entity_velocity, phb) in sync_velocity_query.iter_mut(world) {
            let physics_body = self.physics_bodies.get(&phb).unwrap();
            let velocity = self.bodies.rigid_body_mut(physics_body.body_handle).unwrap()
                .velocity();

            *entity_velocity = velocity.clone();
        }
    }

    pub(crate) fn sync_game_entity_position_to_physics_body(&mut self,
            mut world: &mut legion::world::World,
            physics_body_handle: PhysicsBodyHandle) {
        let sync_position_query = <(Write<Position>, Read<PhysicsBodyHandle>)>::query();

        for (mut pos, phb_comparison) in sync_position_query.iter_mut(world) {
            if physics_body_handle == *phb_comparison {
                self.sync_entity_position_to_physics_position(pos, physics_body_handle);
            }
        }
    }

    fn sync_entity_position_to_physics_position(&mut self, mut pos: legion::borrow::RefMut<Position>, phb: PhysicsBodyHandle) {
        let physics_body = self.physics_bodies.get(&phb).unwrap();
        let trans = self.bodies.rigid_body_mut(physics_body.body_handle).unwrap()
            .position().translation;

        pos.x = trans.x;
        pos.y = trans.y;
    }

    fn sync_physics_position_to_entity_position(&mut self, pos: &Position, phb: PhysicsBodyHandle) {
        let physics_body = self.physics_bodies.get(&phb).unwrap();
        self.bodies.rigid_body_mut(physics_body.body_handle).unwrap()
            .set_position(Isometry2::new(Vector2::new(pos.x, pos.y), std::f32::consts::FRAC_PI_2));
    }
}