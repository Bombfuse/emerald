use crate::{Translation, *};

use crate::core::components::transform::Transform;

use rapier2d::prelude::*;

use crate::crossbeam;
use hecs::{Entity, World};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A physics engine unique to a game world. This handles the RigidBodies of the game.
pub struct PhysicsEngine {
    pub(crate) bodies: RigidBodySet,
    pub(crate) colliders: ColliderSet,
    pub(crate) broad_phase: BroadPhase,
    pub(crate) narrow_phase: NarrowPhase,
    pub(crate) impulse_joints: ImpulseJointSet,
    pub(crate) multibody_joints: MultibodyJointSet,
    pub(crate) island_manager: IslandManager,
    pipeline: PhysicsPipeline,
    pub(crate) gravity: Vector2<f32>,
    pub(crate) ccd_solver: CCDSolver,
    pub(crate) integration_parameters: IntegrationParameters,
    pub(crate) event_handler: ChannelEventCollector,
    pub(crate) collision_event_recv: crossbeam::channel::Receiver<CollisionEvent>,
    pub(crate) contact_force_event_recv: crossbeam::channel::Receiver<ContactForceEvent>,

    entity_bodies: HashMap<Entity, RigidBodyHandle>,
    body_entities: HashMap<RigidBodyHandle, Entity>,
    body_colliders: HashMap<RigidBodyHandle, Vec<ColliderHandle>>,
    collider_body: HashMap<ColliderHandle, RigidBodyHandle>,
    entity_collisions: HashMap<Entity, Vec<Entity>>,
    physics_hooks: Box<dyn PhysicsHooks>,
    query_pipeline: QueryPipeline,
}

impl PhysicsEngine {
    pub(crate) fn new() -> Self {
        let bodies = RigidBodySet::new();
        let colliders = ColliderSet::new();
        let impulse_joints = ImpulseJointSet::new();
        let pipeline = PhysicsPipeline::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let multibody_joints = MultibodyJointSet::new();
        let island_manager = IslandManager::new();

        // Initialize the event collector.
        let (collision_event_send, collision_event_recv) = crossbeam::channel::unbounded();
        let (contact_force_event_send, contact_force_event_recv) = crossbeam::channel::unbounded();
        let event_handler =
            ChannelEventCollector::new(collision_event_send, contact_force_event_send);
        let physics_hooks = Box::new(());
        let ccd_solver = CCDSolver::new();
        let query_pipeline = QueryPipeline::new();

        PhysicsEngine {
            colliders,
            bodies,
            broad_phase,
            narrow_phase,
            impulse_joints,
            multibody_joints,
            pipeline,
            gravity: Vector2::new(0.0, 0.0),
            island_manager,
            ccd_solver,
            integration_parameters: IntegrationParameters::default(),
            collision_event_recv,
            contact_force_event_recv,
            event_handler,
            entity_bodies: HashMap::new(),
            body_entities: HashMap::new(),
            body_colliders: HashMap::new(),
            collider_body: HashMap::new(),
            entity_collisions: HashMap::new(),
            physics_hooks,
            query_pipeline,
        }
    }

    #[inline]
    pub(crate) fn step(&mut self, delta: f32) {
        let dt = self.integration_parameters.dt;
        self.integration_parameters.dt = delta;

        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &(*self.physics_hooks),
            &self.event_handler,
        );

        self.integration_parameters.dt = dt;
    }

    #[inline]
    pub(crate) fn update_query_pipeline(&mut self) {
        self.query_pipeline
            .update(&self.island_manager, &self.bodies, &self.colliders);
    }

    #[inline]
    pub(crate) fn consume_contacts(&mut self) {
        let mut started_contacts = Vec::new();
        let mut stopped_contacts = Vec::new();

        while let Ok(contact_event) = self.collision_event_recv.try_recv() {
            match contact_event {
                CollisionEvent::Started(h1, h2, _flags) => {
                    if let (Some(collider_one), Some(collider_two)) =
                        (self.colliders.get(h1), self.colliders.get(h2))
                    {
                        if let (Some(collider_parent_one), Some(collider_parent_two)) =
                            (collider_one.parent(), collider_two.parent())
                        {
                            if let (Some(entity_one), Some(entity_two)) = (
                                self.body_entities.get(&collider_parent_one),
                                self.body_entities.get(&collider_parent_two),
                            ) {
                                started_contacts.push((*entity_one, *entity_two));
                            }
                        }
                    }
                }
                CollisionEvent::Stopped(h1, h2, _flags) => {
                    if let (Some(collider_one), Some(collider_two)) =
                        (self.colliders.get(h1), self.colliders.get(h2))
                    {
                        if let (Some(collider_parent_one), Some(collider_parent_two)) =
                            (collider_one.parent(), collider_two.parent())
                        {
                            if let (Some(entity_one), Some(entity_two)) = (
                                self.body_entities.get(&collider_parent_one),
                                self.body_entities.get(&collider_parent_two),
                            ) {
                                stopped_contacts.push((*entity_one, *entity_two));
                            }
                        }
                    }
                }
            }
        }

        for started_contact in started_contacts {
            self.add_collision(started_contact.0, started_contact.1);
        }

        for stopped_contact in stopped_contacts {
            self.remove_collision(stopped_contact.0, stopped_contact.1);
        }
    }

    #[inline]
    fn add_collision(&mut self, entity_one: Entity, entity_two: Entity) {
        let entity_one_collisions = self
            .entity_collisions
            .entry(entity_one)
            .or_insert_with(Vec::new);
        entity_one_collisions.push(entity_two);

        let entity_two_collisions = self
            .entity_collisions
            .entry(entity_two)
            .or_insert_with(Vec::new);
        entity_two_collisions.push(entity_one);
    }

    #[inline]
    fn remove_collision(&mut self, entity_one: Entity, entity_two: Entity) {
        if let Some(intersections) = self.entity_collisions.get_mut(&entity_one) {
            intersections.retain(|&x| x != entity_two);
        }

        if let Some(intersections) = self.entity_collisions.get_mut(&entity_two) {
            intersections.retain(|&x| x != entity_one);
        }
    }

    #[inline]
    pub(crate) fn get_colliding_entities(&self, entity: Entity) -> Vec<Entity> {
        if let Some(colliding_bodies) = self.entity_collisions.get(&entity) {
            return colliding_bodies.clone();
        }

        Vec::new()
    }

    #[inline]
    pub fn get_colliders_handles(&self, entity: Entity) -> Vec<ColliderHandle> {
        if let Some(rbh) = self.entity_bodies.get(&entity) {
            if let Some(colliders) = self.body_colliders.get(rbh) {
                return colliders.clone();
            }
        }

        Vec::new()
    }

    #[inline]
    pub fn cast_ray(&mut self, ray_cast_query: RayCastQuery<'_>) -> Option<Entity> {
        if let Some((handle, _toi)) = self.query_pipeline.cast_ray(
            &self.bodies,
            &self.colliders,
            &ray_cast_query.ray,
            ray_cast_query.max_toi,
            ray_cast_query.solid,
            ray_cast_query.filter,
        ) {
            return self.get_entity_from_collider(handle);
        }

        None
    }

    #[inline]
    pub fn cast_shape(
        &self,
        shape: &dyn Shape,
        shape_cast_query: ShapeCastQuery<'_>,
    ) -> Option<Entity> {
        let pos = Isometry::from(Vector2::new(
            shape_cast_query.origin_translation.x,
            shape_cast_query.origin_translation.y,
        ));

        if let Some((handle, _hit)) = self.query_pipeline.cast_shape(
            &self.bodies,
            &self.colliders,
            &pos,
            &shape_cast_query.velocity,
            shape,
            shape_cast_query.max_toi,
            shape_cast_query.stop_at_penetration,
            shape_cast_query.filter,
        ) {
            return self.get_entity_from_collider(handle);
        }

        None
    }

    #[inline]
    fn get_entity_from_collider(&self, collider: ColliderHandle) -> Option<Entity> {
        if let Some(rbh) = self.collider_body.get(&collider) {
            if let Some(entity) = self.body_entities.get(rbh) {
                return Some(entity.clone());
            }
        }

        None
    }

    #[inline]
    pub(crate) fn add_body(
        &mut self,
        entity: Entity,
        body: RigidBody,
        world: &mut World,
    ) -> Result<RigidBodyHandle, EmeraldError> {
        let handle: Result<RigidBodyHandle, EmeraldError> = {
            let handle = self.bodies.insert(body);
            self.entity_bodies.insert(entity, handle);
            self.body_entities.insert(handle, entity);

            Ok(handle)
        };
        let handle = handle?;

        match world.insert(entity, (handle,)) {
            Ok(_) => Ok(handle),
            Err(_e) => {
                self.remove_body(entity);
                Err(EmeraldError::new(
                    "Unable to insert rigid body into entity.",
                ))
            }
        }
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
        let transform = {
            let transform = match world.get::<&Transform>(entity.clone()) {
                Ok(pos) => Ok(pos),
                Err(_e) => Err(EmeraldError::new(
                    "Unable to build a body for an entity without a position",
                )),
            }?;

            (*transform).clone()
        };

        let body = builder
            .translation(Vector2::new(
                transform.translation.x,
                transform.translation.y,
            ))
            .build();
        self.add_body(entity, body, world)
    }

    #[inline]
    pub(crate) fn add_collider(
        &mut self,
        body_handle: RigidBodyHandle,
        collider: Collider,
    ) -> ColliderHandle {
        let handle = self
            .colliders
            .insert_with_parent(collider, body_handle, &mut self.bodies);

        self.collider_body.insert(handle, body_handle);

        if let Some(colliders) = self.body_colliders.get_mut(&body_handle) {
            colliders.push(handle);
        } else {
            self.body_colliders.insert(body_handle, vec![handle]);
        }

        handle
    }

    #[inline]
    pub(crate) fn build_collider(
        &mut self,
        body_handle: RigidBodyHandle,
        builder: ColliderBuilder,
    ) -> ColliderHandle {
        let collider = builder
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();

        self.add_collider(body_handle, collider)
    }

    #[inline]
    pub fn build_joint(
        &mut self,
        parent_handle: RigidBodyHandle,
        child_handle: RigidBodyHandle,
        joint: RevoluteJointBuilder,
        wake_up: bool,
    ) {
        self.impulse_joints
            .insert(parent_handle, child_handle, joint, wake_up);
    }

    #[inline]
    pub(crate) fn remove_body(&mut self, entity: Entity) -> Option<RigidBody> {
        let mut body_entities = Vec::new();
        for e in self.entity_collisions.keys() {
            body_entities.push(*e);
        }

        for body_entity in body_entities {
            self.remove_collision(body_entity, entity);
        }

        if let Some(body_handle) = self.entity_bodies.remove(&entity) {
            self.body_entities.remove(&body_handle);

            if let Some(body) = self.bodies.remove(
                body_handle,
                &mut self.island_manager,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                true,
            ) {
                return Some(body);
            }
        }

        None
    }

    #[inline]
    pub(crate) fn remove_collider(&mut self, collider_handle: ColliderHandle) -> Option<Collider> {
        if let Some(collider) = self.colliders.remove(
            collider_handle,
            &mut self.island_manager,
            &mut self.bodies,
            false,
        ) {
            if let Some(rbh) = self.collider_body.remove(&collider_handle) {
                if let Some(colliders) = self.body_colliders.get_mut(&rbh) {
                    let mut i = 0;
                    let mut found = false;

                    for c in colliders.iter() {
                        if *c == collider_handle {
                            found = true;
                            break;
                        }

                        i += 1;
                    }

                    if found {
                        colliders.remove(i);
                    }
                }
            }

            return Some(collider);
        }

        None
    }

    #[inline]
    pub(crate) fn sync_physics_world_to_game_world(&mut self, world: &mut hecs::World) {
        for (_id, (transform, rbh)) in world.query::<(&mut Transform, &RigidBodyHandle)>().iter() {
            self.sync_physics_position_to_entity_position(transform, *rbh);
        }
    }

    #[inline]
    pub(crate) fn sync_game_world_to_physics_world(&mut self, world: &mut hecs::World) {
        for (_id, (transform, rbh)) in world.query::<(&mut Transform, &RigidBodyHandle)>().iter() {
            self.sync_entity_position_to_physics_position(transform, *rbh);
        }
    }

    #[inline]
    fn sync_entity_position_to_physics_position(
        &mut self,
        transform: &mut Transform,
        body_handle: RigidBodyHandle,
    ) {
        if let Some(body_transform) = self.bodies.get(body_handle) {
            let translation = body_transform.position().translation;
            transform.translation = Translation::new(translation.x, translation.y);
        }
    }

    #[inline]
    fn sync_physics_position_to_entity_position(
        &mut self,
        transform: &Transform,
        body_handle: RigidBodyHandle,
    ) {
        if let Some(body) = self.bodies.get_mut(body_handle) {
            if body.is_kinematic() {
                body.set_next_kinematic_position(Isometry::from(Vector2::new(
                    transform.translation.x,
                    transform.translation.y,
                )))
            } else {
                body.set_translation(
                    Vector2::new(transform.translation.x, transform.translation.y),
                    true,
                );
            }
        }
    }
}
