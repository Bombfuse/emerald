use crate::*;

use rapier2d::na::Isometry2;
use rapier2d::dynamics::{JointSet, RigidBodyBuilder, RigidBodySet, RigidBodyHandle, IntegrationParameters};
use rapier2d::geometry::{BroadPhase, NarrowPhase, ColliderBuilder, ColliderSet, ColliderHandle, ContactEvent, ProximityEvent};
use rapier2d::pipeline::{ChannelEventCollector, PhysicsPipeline};

use hecs::{Entity, World};
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
    pub(crate) event_handler: ChannelEventCollector,
    pub(crate) contact_recv: crossbeam::channel::Receiver<ContactEvent>,
    pub(crate) proximity_recv: crossbeam::channel::Receiver<ProximityEvent>,
    pub(crate) entity_bodies: HashMap<Entity, RigidBodyHandle>,
}

impl PhysicsEngine {
    pub(crate) fn new() -> Self {
        let bodies = RigidBodySet::new();
        let colliders = ColliderSet::new();
        let joints = JointSet::new();
        let pipeline = PhysicsPipeline::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        
        // Initialize the event collector.
        let (contact_send, contact_recv) = crossbeam::channel::unbounded();
        let (proximity_send, proximity_recv) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(proximity_send, contact_send);

        PhysicsEngine {
            colliders,
            bodies,
            broad_phase,
            narrow_phase,
            joints,
            pipeline,
            gravity: Vector2::new(0.0, 0.0),
            integration_parameters: IntegrationParameters::default(),
            contact_recv,
            proximity_recv,
            event_handler,
            entity_bodies: HashMap::new(),
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
            &mut self.event_handler,
        )
    }

    pub(crate) fn try_recv_proximity(&mut self) -> Result<ProximityEvent, EmeraldError> { Ok(self.proximity_recv.try_recv()?) }
    pub(crate) fn try_recv_contact(&mut self) -> Result<ContactEvent, EmeraldError> { Ok(self.contact_recv.try_recv()?) }

    /// Builds the described rigid body for the given entity.
    /// Fails if the entity does not have a position or if the handle is unable to be inserted into the ECS world.
    pub(crate) fn build_body(&mut self, entity: Entity, builder: RigidBodyBuilder, world: &mut World) -> Result<RigidBodyHandle, EmeraldError> {
        let handle: Result<RigidBodyHandle, EmeraldError> = {
            let position = match world.get::<Position>(entity.clone()) {
                Ok(pos) => Ok(pos),
                Err(_e) => Err(EmeraldError::new("Unable to build a body for an entity without a position"))
            }?;

            let body = builder.translation(position.x, position.y).build();
            let handle = self.bodies.insert(body);
            self.entity_bodies.insert(entity.clone(), handle);
            
            Ok(handle)
        };
        let handle = handle?;

        match world.insert(entity.clone(), (handle.clone(),)) {
            Ok(_) => Ok(handle),
            Err(_e) => {
                self.remove_body(entity);
                Err(EmeraldError::new("Unable to insert rigid body into entity."))
            }
        }

    }

    pub(crate) fn build_collider(&mut self, body_handle: RigidBodyHandle, builder: ColliderBuilder) -> ColliderHandle {
        let collider = builder.build();
        let handle = self.colliders.insert(collider, body_handle, &mut self.bodies);

        handle
    }

    pub(crate) fn remove_body(&mut self, entity: Entity) -> Option<RigidBody> {
        if let Some(body_handle) = self.entity_bodies.get(&entity) {
            if let Some(body) = self.pipeline.remove_rigid_body(
                *body_handle,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.bodies,
                &mut self.colliders,
                &mut self.joints
            ) {
                self.entity_bodies.remove(&entity);
    
                return Some(body);
            }
        }

        None
    }

    pub(crate) fn remove_collider(&mut self, collider_handle: ColliderHandle) -> Option<Collider> {
        self.pipeline.remove_collider(
            collider_handle,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders
        )
    }

    pub(crate) fn sync_physics_world_to_game_world(&mut self, world: &mut hecs::World) {
        for (_id, (pos, rbh)) in world.query::<(&mut Position, &RigidBodyHandle)>().iter() {
            self.sync_physics_position_to_entity_position(&pos, *rbh);
        }
    }

    pub(crate) fn sync_game_world_to_physics_world(&mut self, world: &mut hecs::World) {
        for (_id, (pos, rbh)) in world.query::<(&mut Position, &RigidBodyHandle)>().iter() {
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