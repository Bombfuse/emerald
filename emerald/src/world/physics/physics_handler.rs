use crate::physics::*;
use crate::{EmeraldError, Vector2};

use hecs::Entity;
use rapier2d::prelude::*;

pub struct PhysicsHandler<'a> {
    physics_engine: &'a mut PhysicsEngine,
    world: &'a mut hecs::World,
}
impl<'a> PhysicsHandler<'a> {
    pub fn new(physics_engine: &'a mut PhysicsEngine, world: &'a mut hecs::World) -> Self {
        PhysicsHandler {
            world,
            physics_engine,
        }
    }

    pub fn build_body(
        &mut self,
        entity: Entity,
        desc: RigidBodyBuilder,
    ) -> Result<RigidBodyHandle, EmeraldError> {
        self.physics_engine
            .build_body(entity, desc, &mut self.world)
    }

    pub fn build_collider(
        &mut self,
        body_handle: RigidBodyHandle,
        desc: ColliderBuilder,
    ) -> ColliderHandle {
        self.physics_engine.build_collider(body_handle, desc)
    }

    pub fn build_joint(
        &mut self,
        parent_handle: RigidBodyHandle,
        child_handle: RigidBodyHandle,
        joint: RevoluteJointBuilder,
        wake_up: bool,
    ) {
        self.physics_engine
            .build_joint(parent_handle, child_handle, joint, wake_up)
    }

    /// Retrieves the entities with bodies that are touching the body of this entity.
    /// This includes:
    /// Collider <- Contact -> Collider
    /// Collider <- Contact -> Sensor
    /// Sensor <- Contact -> Sensor
    pub fn get_colliding_entities(&self, entity: Entity) -> Vec<Entity> {
        self.physics_engine.get_colliding_entities(entity)
    }

    #[deprecated(note = "use `world.physics().get_colliding_entities()` instead")]
    /// Retrieves the entities with bodies that are touching the rigid body of this entity.
    pub fn get_colliding_bodies(&self, entity: Entity) -> Vec<Entity> {
        self.physics_engine.get_colliding_entities(entity)
    }

    #[deprecated(note = "use `world.physics().get_colliding_entities()` instead")]
    /// Retrieves the entities with sensors that are touching this entity.
    pub fn get_colliding_areas(&self, entity: Entity) -> Vec<Entity> {
        self.physics_engine.get_colliding_entities(entity)
    }

    pub fn get_collider_handles(&self, entity: Entity) -> Vec<ColliderHandle> {
        self.physics_engine.get_colliders_handles(entity)
    }

    pub fn get_collider(&self, collider_handle: ColliderHandle) -> Option<&Collider> {
        self.physics_engine.colliders.get(collider_handle)
    }

    pub fn get_collider_mut(&mut self, collider_handle: ColliderHandle) -> Option<&mut Collider> {
        self.physics_engine.colliders.get_mut(collider_handle)
    }

    /// Remove physics body attached to this entity.
    pub fn remove_body(&mut self, entity: Entity) -> Option<RigidBody> {
        if let Some(body) = self.physics_engine.remove_body(entity) {
            if self.world.remove_one::<RigidBodyHandle>(entity).is_ok() {
                return Some(body);
            }
        }

        None
    }

    pub fn remove_collider(&mut self, collider_handle: ColliderHandle) -> Option<Collider> {
        self.physics_engine.remove_collider(collider_handle)
    }

    pub fn rigid_body(&mut self, body_handle: RigidBodyHandle) -> Option<&RigidBody> {
        self.physics_engine.bodies.get(body_handle)
    }

    pub fn rigid_body_mut(&mut self, body_handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.physics_engine.bodies.get_mut(body_handle)
    }

    pub fn body_count(&self) -> usize {
        self.physics_engine.bodies.len()
    }

    /// Returns the first entity the ray hits if one exists.
    pub fn cast_ray(&mut self, ray_cast_query: RayCastQuery<'_>) -> Option<Entity> {
        self.physics_engine.cast_ray(ray_cast_query)
    }

    /// Returns the first entity the shape collides with if one exists.
    pub fn cast_shape(
        &self,
        shape: &dyn Shape,
        shape_cast_query: ShapeCastQuery<'_>,
    ) -> Option<Entity> {
        self.physics_engine.cast_shape(shape, shape_cast_query)
    }

    /// Steps the physics at 1/60 timestep
    pub fn step(&mut self, delta: f32) {
        self.step_n(1, delta);
    }

    /// Steps the physics n-times at 1/60 timestep
    pub fn step_n(&mut self, n: u32, delta: f32) {
        self.physics_engine
            .sync_physics_world_to_game_world(&mut self.world);

        for _ in 0..n {
            self.physics_engine.step(delta);
        }

        self.physics_engine.consume_contacts();
        self.physics_engine.update_query_pipeline();

        self.physics_engine
            .sync_game_world_to_physics_world(&mut self.world);
    }

    pub fn set_gravity(&mut self, gravity: Vector2<f32>) {
        self.physics_engine.gravity = gravity;
    }
}
