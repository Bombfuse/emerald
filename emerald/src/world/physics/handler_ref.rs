use crate::physics::*;

use hecs::Entity;

pub struct PhysicsRefHandler<'a> {
    physics_engine: &'a PhysicsEngine,
}
impl<'a> PhysicsRefHandler<'a> {
    pub(crate) fn new(physics_engine: &'a PhysicsEngine) -> Self {
        PhysicsRefHandler { physics_engine }
    }

    /// Retrieves the entities with bodies that are touching the body of this entity.
    /// This includes:
    /// Collider <- Contact -> Collider
    /// Collider <- Contact -> Sensor
    /// Sensor <- Contact -> Sensor
    pub fn get_colliding_entities(&self, entity: Entity) -> Vec<Entity> {
        self.physics_engine.get_colliding_entities(entity)
    }

    /// Retrieves the entities with bodies that are touching the body of this entity.
    pub fn get_colliding_bodies(&self, entity: Entity) -> Vec<Entity> {
        self.physics_engine.get_colliding_entities(entity)
    }

    /// Retrieves the entities with sensors that are touching this entity.
    pub fn get_colliding_areas(&self, entity: Entity) -> Vec<Entity> {
        self.physics_engine.get_colliding_entities(entity)
    }
}
