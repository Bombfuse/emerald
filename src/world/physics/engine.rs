use crate::*;
use rapier2d::dynamics::{JointSet, RigidBodyBuilder, RigidBodySet, RigidBodyHandle, IntegrationParameters};
use rapier2d::geometry::{BroadPhase, NarrowPhase, ColliderBuilder, ColliderSet, ColliderHandle};
use rapier2d::na::Isometry2;
use rapier2d::pipeline::{ChannelEventCollector, PhysicsPipeline};

use std::collections::HashMap;

/// A physics engine unique to a game world. This handles the RigidBodies of the game.
pub struct PhysicsEngine {
    pub(crate) bodies: RigidBodySet,
    pub(crate) colliders: ColliderSet,
    pub(crate) broad_phase: BroadPhase,
    pub(crate) narrow_phase: NarrowPhase,
    pub(crate) joints: JointSet,
    pipeline: PhysicsPipeline,
    pub(crate) gravity: Vector2<f32>,
    pub(crate) integration_parameters: IntegrationParameters,
    // pub(crate) event_handler: ChannelEventCollector,
}

impl PhysicsEngine {
    pub(crate) fn new() -> Self {
        let bodies = RigidBodySet::new();
        let colliders = ColliderSet::new();
        let joints = JointSet::new();
        let pipeline = PhysicsPipeline::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();

        PhysicsEngine {
            colliders,
            bodies,
            broad_phase,
            narrow_phase,
            joints,
            pipeline,
            gravity: Vector2::new(0.0, 0.0),
            integration_parameters: IntegrationParameters::default(),
        }
    }

    pub(crate) fn step(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &(),
        )
    }

    pub(crate) fn create_body(&mut self, builder: &RigidBodyBuilder) -> RigidBodyHandle {
        let body = builder.build();
        let handle = self.bodies.insert(body);

        handle
    }

    pub(crate) fn create_collider(&mut self, body_handle: RigidBodyHandle, builder: &ColliderBuilder) -> ColliderHandle {
        let collider = builder.build();
        let handle = self.colliders.insert(collider, body_handle, &mut self.bodies);

        handle
    }

    pub(crate) fn sync_physics_world_to_game_world(&mut self, world: &mut hecs::World) {
        for (id, (pos, rbh)) in world.query::<(&mut Position, &RigidBodyHandle)>().iter() {
            self.sync_physics_position_to_entity_position(&pos, *rbh);
        }
    }

    pub(crate) fn sync_game_world_to_physics_world(&mut self, world: &mut hecs::World) {
        for (id, (pos, rbh)) in world.query::<(&mut Position, &RigidBodyHandle)>().iter() {
            self.sync_entity_position_to_physics_position(pos, *rbh);
        }
    }

    fn sync_entity_position_to_physics_position(&mut self, mut pos: &mut Position, body_handle: RigidBodyHandle) {
        let trans = self.bodies.get_mut(body_handle).unwrap()
            .position.translation;

        pos.x = trans.x;
        pos.y = trans.y;
    }

    fn sync_physics_position_to_entity_position(&mut self, pos: &Position, body_handle: RigidBodyHandle) {
        self.bodies.get_mut(body_handle).unwrap()
            .position = Isometry2::translation(pos.x, pos.y);
    }
}