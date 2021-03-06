use crate::*;

use rapier2d::dynamics::{
    BodyStatus, IntegrationParameters, JointSet, RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};
use rapier2d::geometry::{
    BroadPhase, ColliderBuilder, ColliderHandle, ColliderSet, ContactEvent, IntersectionEvent,
    NarrowPhase,
};
use rapier2d::na::Isometry2;
use rapier2d::pipeline::{ChannelEventCollector, PhysicsPipeline};

use crate::crossbeam;
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
    pub(crate) intersection_recv: crossbeam::channel::Receiver<IntersectionEvent>,

    entity_bodies: HashMap<Entity, RigidBodyHandle>,
    body_entities: HashMap<RigidBodyHandle, Entity>,
    entity_collisions: HashMap<Entity, Vec<Entity>>,
    entity_intersections: HashMap<Entity, Vec<Entity>>,
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
        let (intersection_send, intersection_recv) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(intersection_send, contact_send);

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
            intersection_recv,
            event_handler,
            entity_bodies: HashMap::new(),
            body_entities: HashMap::new(),
            entity_collisions: HashMap::new(),
            entity_intersections: HashMap::new(),
        }
    }

    #[inline]
    pub(crate) fn step(&mut self, delta: f32) {
        let dt = self.integration_parameters.dt;
        self.integration_parameters.dt = delta;

        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            None,
            None,
            &mut self.event_handler,
        );

        self.integration_parameters.dt = dt;
    }

    #[inline]
    pub(crate) fn consume_contacts(&mut self) {
        let mut started_contacts = Vec::new();
        let mut stopped_contacts = Vec::new();

        while let Ok(contact_event) = self.contact_recv.try_recv() {
            match contact_event {
                ContactEvent::Started(h1, h2) => {
                    if let (Some(collider_one), Some(collider_two)) =
                        (self.colliders.get(h1), self.colliders.get(h2))
                    {
                        if let (Some(entity_one), Some(entity_two)) = (
                            self.body_entities.get(&collider_one.parent()),
                            self.body_entities.get(&collider_two.parent()),
                        ) {
                            started_contacts.push((*entity_one, *entity_two));
                        }
                    }
                }
                ContactEvent::Stopped(h1, h2) => {
                    if let (Some(collider_one), Some(collider_two)) =
                        (self.colliders.get(h1), self.colliders.get(h2))
                    {
                        if let (Some(entity_one), Some(entity_two)) = (
                            self.body_entities.get(&collider_one.parent()),
                            self.body_entities.get(&collider_two.parent()),
                        ) {
                            stopped_contacts.push((*entity_one, *entity_two));
                        }
                    }
                }
            }
        }

        for started_contact in started_contacts {
            self.add_body_contact(started_contact.0, started_contact.1);
        }

        for stopped_contact in stopped_contacts {
            self.remove_body_contact(stopped_contact.0, stopped_contact.1);
        }
    }

    #[inline]
    pub(crate) fn consume_intersections(&mut self) {
        let mut intersections = Vec::new();
        let mut disjoints = Vec::new();

        while let Ok(intersection_event) = self.intersection_recv.try_recv() {
            if let (Some(collider_one), Some(collider_two)) = (
                self.colliders.get(intersection_event.collider1),
                self.colliders.get(intersection_event.collider2),
            ) {
                if let (Some(entity_one), Some(entity_two)) = (
                    self.body_entities.get(&collider_one.parent()),
                    self.body_entities.get(&collider_two.parent()),
                ) {
                    if intersection_event.intersecting {
                        intersections.push((*entity_one, *entity_two));
                    } else {
                        disjoints.push((*entity_one, *entity_two));
                    }
                }
            }
        }

        for intersection in intersections {
            self.add_sensor_intersection(intersection.0, intersection.1);
        }

        for disjoint in disjoints {
            self.remove_sensor_intersection(disjoint.0, disjoint.1);
        }
    }

    #[inline]
    fn add_sensor_intersection(&mut self, entity_one: Entity, entity_two: Entity) {
        let entity_one_collisions = self
            .entity_intersections
            .entry(entity_one)
            .or_insert(Vec::new());
        entity_one_collisions.push(entity_two.clone());

        let entity_two_collisions = self
            .entity_intersections
            .entry(entity_two)
            .or_insert(Vec::new());
        entity_two_collisions.push(entity_one.clone());
    }

    #[inline]
    fn remove_sensor_intersection(&mut self, entity_one: Entity, entity_two: Entity) {
        if let Some(intersections) = self.entity_intersections.get_mut(&entity_one) {
            intersections.retain(|&x| x != entity_two);
        }

        if let Some(intersections) = self.entity_intersections.get_mut(&entity_two) {
            intersections.retain(|&x| x != entity_one);
        }
    }

    #[inline]
    fn add_body_contact(&mut self, entity_one: Entity, entity_two: Entity) {
        let entity_one_collisions = self
            .entity_collisions
            .entry(entity_one)
            .or_insert(Vec::new());
        entity_one_collisions.push(entity_two.clone());

        let entity_two_collisions = self
            .entity_collisions
            .entry(entity_two)
            .or_insert(Vec::new());
        entity_two_collisions.push(entity_one.clone());
    }

    #[inline]
    fn remove_body_contact(&mut self, entity_one: Entity, entity_two: Entity) {
        if let Some(intersections) = self.entity_collisions.get_mut(&entity_one) {
            intersections.retain(|&x| x != entity_two);
        }

        if let Some(intersections) = self.entity_collisions.get_mut(&entity_two) {
            intersections.retain(|&x| x != entity_one);
        }
    }

    #[inline]
    pub(crate) fn get_colliding_areas(&self, entity: Entity) -> Vec<Entity> {
        if let Some(colliding_areas) = self.entity_intersections.get(&entity) {
            return colliding_areas.clone();
        }

        Vec::new()
    }

    #[inline]
    pub(crate) fn get_colliding_bodies(&self, entity: Entity) -> Vec<Entity> {
        if let Some(colliding_bodies) = self.entity_collisions.get(&entity) {
            return colliding_bodies.clone();
        }

        Vec::new()
    }

    /// Builds the described rigid body for the given entity.
    /// Fails if the entity does not have a position or if the handle is unable to be inserted into the ECS world.
    #[inline]
    pub(crate) fn build_body(
        &mut self,
        entity: Entity,
        builder: RigidBodyBuilder,
        world: &mut World,
    ) -> Result<RigidBodyHandle, EmeraldError> {
        let handle: Result<RigidBodyHandle, EmeraldError> = {
            let position = match world.get::<Position>(entity.clone()) {
                Ok(pos) => Ok(pos),
                Err(_e) => Err(EmeraldError::new(
                    "Unable to build a body for an entity without a position",
                )),
            }?;

            let body = builder.translation(position.x, position.y).build();
            let handle = self.bodies.insert(body);
            self.entity_bodies.insert(entity.clone(), handle);
            self.body_entities.insert(handle, entity);

            Ok(handle)
        };
        let handle = handle?;

        match world.insert(entity.clone(), (handle.clone(),)) {
            Ok(_) => Ok(handle),
            Err(_e) => {
                self.remove_body(entity);
                Err(EmeraldError::new(
                    "Unable to insert rigid body into entity.",
                ))
            }
        }
    }

    #[inline]
    pub(crate) fn build_collider(
        &mut self,
        body_handle: RigidBodyHandle,
        builder: ColliderBuilder,
    ) -> ColliderHandle {
        let collider = builder.build();
        let handle = self
            .colliders
            .insert(collider, body_handle, &mut self.bodies);

        handle
    }

    #[inline]
    pub(crate) fn remove_body(&mut self, entity: Entity) -> Option<RigidBody> {
        let mut body_entities = Vec::new();
        for (e, _) in &self.entity_collisions {
            body_entities.push(e.clone());
        }

        for body_entity in body_entities {
            self.remove_body_contact(body_entity, entity.clone());
        }

        let mut sensor_entities = Vec::new();
        for (e, _) in &self.entity_intersections {
            sensor_entities.push(e.clone());
        }
        for sensor_entity in sensor_entities {
            self.remove_sensor_intersection(entity.clone(), sensor_entity);
        }

        if let Some(body_handle) = self.entity_bodies.remove(&entity) {
            self.body_entities.remove(&body_handle);

            if let Some(body) =
                self.bodies
                    .remove(body_handle, &mut self.colliders, &mut self.joints)
            {
                return Some(body);
            }
        }

        None
    }

    #[inline]
    pub(crate) fn remove_collider(&mut self, collider_handle: ColliderHandle) -> Option<Collider> {
        self.colliders
            .remove(collider_handle, &mut self.bodies, false)
    }

    #[inline]
    pub(crate) fn sync_physics_world_to_game_world(&mut self, world: &mut hecs::World) {
        for (_id, (pos, rbh)) in world.query::<(&mut Position, &RigidBodyHandle)>().iter() {
            self.sync_physics_position_to_entity_position(&pos, *rbh);
        }
    }

    #[inline]
    pub(crate) fn sync_game_world_to_physics_world(&mut self, world: &mut hecs::World) {
        for (_id, (pos, rbh)) in world.query::<(&mut Position, &RigidBodyHandle)>().iter() {
            self.sync_entity_position_to_physics_position(pos, *rbh);
        }
    }

    #[inline]
    fn sync_entity_position_to_physics_position(
        &mut self,
        mut pos: &mut Position,
        body_handle: RigidBodyHandle,
    ) {
        if let Some(transform) = self.bodies.get(body_handle) {
            let translation = transform.position().translation;
            pos.x = translation.x;
            pos.y = translation.y;
        }
    }

    #[inline]
    fn sync_physics_position_to_entity_position(
        &mut self,
        pos: &Position,
        body_handle: RigidBodyHandle,
    ) {
        if let Some(body) = self.bodies.get_mut(body_handle) {
            match body.body_status {
                BodyStatus::Kinematic => {
                    body.set_next_kinematic_position(Isometry2::translation(pos.x, pos.y))
                }
                _ => body.set_position(Isometry2::translation(pos.x, pos.y), false),
            }
        }
    }
}
