pub mod physics;

pub mod ent;
pub mod navigation;
pub mod world_physics_loader;

use std::collections::HashMap;

use crate::{
    rendering::components::Camera, AssetLoader, EmeraldError, PhysicsEngine, PhysicsHandler,
    PhysicsRefHandler, Transform,
};

use hecs::{
    Bundle, Component, ComponentRef, DynamicBundle, Entity, NoSuchEntity, Query, QueryBorrow,
    QueryItem, QueryOne, SpawnBatchIter,
};
use rapier2d::prelude::{RigidBodyBuilder, RigidBodyHandle};

use self::{
    ent::{load_ent, EntLoadConfig},
    world_physics_loader::load_world_physics,
};

pub struct World {
    pub(crate) physics_engine: PhysicsEngine,
    pub(crate) inner: hecs::World,
}
impl Default for World {
    fn default() -> Self {
        World {
            physics_engine: PhysicsEngine::new(),
            inner: hecs::World::default(),
        }
    }
}
impl World {
    pub fn new() -> Self {
        Default::default()
    }

    // TODO: Make entity ids, rigid body handles, and collider handles unique across all worlds. Then we can remove the HashMap of OldEntity -> NewEntity.
    /// Absorbs another world into this one. Resets and changes the Entity ids of the other worlds, when they are merged into this world.
    /// All entities are placed into this world at their current transform.
    /// The camera of the primary world will remain the current camera.
    /// If physics is enabled, will keep its own physics settings.
    /// Returns a map of OldEntity -> NewEntity. If you have components that store Entity references, use this map to update your references.
    pub fn merge(
        &mut self,
        mut other_world: World,
    ) -> Result<HashMap<Entity, Entity>, EmeraldError> {
        let mut entity_id_shift_map = HashMap::new();
        let other_entities = other_world
            .inner
            .iter()
            .map(|entity_ref| entity_ref.entity())
            .collect::<Vec<Entity>>();

        for old_id in other_entities {
            match other_world.inner.take(old_id.clone()) {
                Err(_) => {
                    return Err(EmeraldError::new(format!(
                        "Entity {:?} does not exist, cannot merge.",
                        old_id
                    )))
                }
                Ok(bundle) => {
                    let new_id = self.inner.spawn(bundle);
                    entity_id_shift_map.insert(old_id.clone(), new_id.clone());
                    self.merge_physics_entity(&mut other_world.physics_engine, old_id, new_id)?;
                }
            }
        }

        Ok(entity_id_shift_map)
    }

    /// Helper function for [`merge`]
    fn merge_physics_entity(
        &mut self,
        other_world_physics: &mut PhysicsEngine,
        old_id: Entity,
        new_id: Entity,
    ) -> Result<(), EmeraldError> {
        let mut colliders = Vec::new();
        for c_id in other_world_physics.get_colliders_handles(old_id.clone()) {
            if let Some(collider) = other_world_physics.remove_collider(c_id) {
                colliders.push(collider);
            }
        }

        if let Some(rigid_body) = other_world_physics.remove_body(old_id.clone()) {
            let new_rbh =
                self.physics_engine
                    .add_body(new_id.clone(), rigid_body, &mut self.inner)?;

            for collider in colliders {
                self.physics_engine.add_collider(new_rbh, collider);
            }
        }

        Ok(())
    }

    /// Disable all cameras then set the camera on the given entity as active.
    /// Fails if the given entity does not exist, or does not have a camera.
    #[inline]
    pub fn make_active_camera(&mut self, entity: Entity) -> Result<(), EmeraldError> {
        let mut set_camera = false;
        if let Ok(mut camera) = self.get::<&mut Camera>(entity) {
            camera.is_active = true;
            set_camera = true;
        }

        if set_camera {
            for (id, mut camera_to_disable) in self.query::<&mut Camera>().iter() {
                if id != entity {
                    camera_to_disable.is_active = false;
                }
            }

            return Ok(());
        }

        Err(EmeraldError::new(format!(
            "Entity {:?} either does not exist or does not hold a camera",
            entity
        )))
    }

    #[inline]
    pub fn get_active_camera(&self) -> Option<Entity> {
        let mut cam = None;

        for (id, camera) in self.query::<&Camera>().iter() {
            if camera.is_active {
                cam = Some(id);
                break;
            }
        }

        cam
    }

    pub fn spawn(&mut self, components: impl DynamicBundle) -> Entity {
        self.inner.spawn(components)
    }

    pub fn spawn_with_body(
        &mut self,
        components: impl DynamicBundle,
        body_builder: RigidBodyBuilder,
    ) -> Result<(Entity, RigidBodyHandle), EmeraldError> {
        let entity = self.spawn(components);
        let rbh = self.physics().build_body(entity, body_builder)?;

        Ok((entity, rbh))
    }

    pub fn spawn_batch<I>(&mut self, iter: I) -> SpawnBatchIter<'_, I::IntoIter>
    where
        I: IntoIterator,
        I::Item: Bundle + 'static,
    {
        self.inner.spawn_batch::<I>(iter)
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), EmeraldError> {
        self.physics_engine.remove_body(entity);

        match self.inner.despawn(entity.clone()) {
            Ok(()) => Ok(()),
            Err(e) => Err(EmeraldError::new(format!(
                "Error despawning entity {:?}. {:?}",
                entity, e
            ))),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.physics_engine = PhysicsEngine::new();
    }

    pub fn query<Q: Query>(&self) -> QueryBorrow<'_, Q> {
        self.inner.query::<Q>()
    }

    pub fn get<'a, T: ComponentRef<'a>>(&'a self, entity: Entity) -> Result<T::Ref, EmeraldError> {
        match self.inner.get::<T>(entity.clone()) {
            Ok(component_ref) => Ok(component_ref),
            Err(e) => Err(EmeraldError::new(format!(
                "Error getting component for entity {:?}. {:?}",
                entity, e
            ))),
        }
    }

    pub fn reserve_entity(&self) -> Entity {
        self.inner.reserve_entity()
    }

    /// Whether `entity` still exists
    pub fn contains(&self, entity: Entity) -> bool {
        self.inner.contains(entity)
    }

    pub fn count(&self) -> usize {
        self.inner.len() as usize
    }

    /// Prepare a query against a single entity, using dynamic borrow checking
    ///
    /// Prefer [`query_one_mut`](Self::query_one_mut) when concurrent access to the [`World`] is not
    /// required.
    ///
    /// Call [`get`](QueryOne::get) on the resulting [`QueryOne`] to actually execute the query. The
    /// [`QueryOne`] value is responsible for releasing the dynamically-checked borrow made by
    /// `get`, so it can't be dropped while references returned by `get` are live.
    ///
    /// Handy for accessing multiple components simultaneously.
    pub fn query_one<Q: Query>(&self, entity: Entity) -> Result<QueryOne<'_, Q>, EmeraldError> {
        match self.inner.query_one::<Q>(entity.clone()) {
            Ok(component_ref) => Ok(component_ref),
            Err(e) => Err(EmeraldError::new(format!(
                "Error querying for entity {:?}. {:?}",
                entity, e
            ))),
        }
    }

    /// Query a single entity in a uniquely borrow world
    ///
    /// Like [`query_one`](Self::query_one), but faster because dynamic borrow checks can be
    /// skipped. Note that, unlike [`query_one`](Self::query_one), on success this returns the
    /// query's results directly.
    pub fn query_one_mut<Q: Query>(
        &mut self,
        entity: Entity,
    ) -> Result<QueryItem<'_, Q>, EmeraldError> {
        match self.inner.query_one_mut::<Q>(entity.clone()) {
            Ok(component_ref) => Ok(component_ref),
            Err(e) => Err(EmeraldError::new(format!(
                "Error querying for entity {:?}. {:?}",
                entity, e
            ))),
        }
    }

    pub fn insert_one(
        &mut self,
        entity: Entity,
        component: impl Component,
    ) -> Result<(), NoSuchEntity> {
        self.inner.insert(entity, (component,))
    }

    pub fn insert(
        &mut self,
        entity: Entity,
        components: impl DynamicBundle,
    ) -> Result<(), NoSuchEntity> {
        self.inner.insert(entity, components)
    }

    pub fn remove<T: Bundle + 'static>(&mut self, entity: Entity) -> Result<T, EmeraldError> {
        match self.inner.remove::<T>(entity.clone()) {
            Ok(removed_bundle) => Ok(removed_bundle),
            Err(e) => Err(EmeraldError::new(format!(
                "Error removing bundle for entity {:?}. {:?}",
                entity, e
            ))),
        }
    }

    pub fn remove_one<T: Component>(&mut self, entity: Entity) -> Result<T, EmeraldError> {
        match self.inner.remove_one::<T>(entity.clone()) {
            Ok(removed_component) => Ok(removed_component),
            Err(e) => Err(EmeraldError::new(format!(
                "Error removing component for entity {:?}. {:?}",
                entity, e
            ))),
        }
    }

    pub fn physics(&mut self) -> PhysicsHandler<'_> {
        PhysicsHandler::new(&mut self.physics_engine, &mut self.inner)
    }

    pub fn physics_ref(&self) -> PhysicsRefHandler<'_> {
        PhysicsRefHandler::new(&self.physics_engine)
    }
}

pub struct WorldLoadConfig<'a> {
    pub transform_offset: Transform,
    pub custom_component_loader: Option<
        &'a dyn Fn(
            &mut AssetLoader<'_>,
            Entity,
            &mut World,
            toml::Value,
            String,
        ) -> Result<(), EmeraldError>,
    >,
}
impl<'a> Default for WorldLoadConfig<'a> {
    fn default() -> Self {
        Self {
            transform_offset: Default::default(),
            custom_component_loader: None,
        }
    }
}

const PHYSICS_SCHEMA_KEY: &str = "physics";
const ENTITIES_SCHEMA_KEY: &str = "entities";

pub(crate) fn load_world(
    loader: &mut AssetLoader<'_>,
    toml: String,
    config: WorldLoadConfig<'_>,
) -> Result<World, EmeraldError> {
    let mut toml = toml.parse::<toml::Value>()?;
    let mut world = World::new();

    if let Some(table) = toml.as_table_mut() {
        // TODO: set physics here
        if let Some(physics_val) = table.remove(PHYSICS_SCHEMA_KEY) {
            load_world_physics(loader, &mut world, &physics_val)?;
        }

        if let Some(mut entities_val) = table.remove(ENTITIES_SCHEMA_KEY) {
            if let Some(entities) = entities_val.as_array_mut() {
                for value in entities {
                    let config = EntLoadConfig {
                        transform: Transform::default(),
                        custom_component_loader: config.custom_component_loader,
                    };

                    // check if this is a ent path reference
                    if let Some(path) = value.get("path") {
                        if let Some(path) = path.as_str() {
                            loader.ent(&mut world, config, path)?;
                        }
                    } else {
                        load_ent(loader, &mut world, value, config)?;
                    }
                }
            }
        }
    }

    Ok(world)
}

#[cfg(test)]
mod tests {
    use crate::{rendering::components::Camera, Transform, World};

    #[test]
    fn make_active_camera_succeeds_on_entity_with_camera() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(), Camera::default()));
        assert!(world.make_active_camera(entity).is_ok());

        let active_camera_entity = world.get_active_camera().unwrap();

        assert_eq!(entity, active_camera_entity);
    }

    #[test]
    fn make_active_camera_fails_on_entity_without_camera() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));

        assert!(world.make_active_camera(entity).is_err());
    }

    #[test]
    fn get_succeeds_on_preexisting_component() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));

        assert!(world.get::<&Transform>(entity).is_ok());
    }

    #[test]
    fn get_fails_on_nonexisting_component() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));

        assert!(world.get::<&String>(entity).is_err());
    }

    #[test]
    fn get_can_mutate() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));
        let expected_value = 100.0;

        {
            let mut entity_transform = world.get::<&mut Transform>(entity).unwrap();
            entity_transform.translation.x = expected_value;
        }

        assert!(world.get::<&Transform>(entity).unwrap().translation.x == expected_value);
    }

    #[test]
    fn query_one_is_none_on_nonexisting_bundle() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));

        let mut query = world.query_one::<(&Transform, &String)>(entity).unwrap();
        assert!(query.get().is_none());
    }

    #[test]
    fn query_one_is_some_on_existing_bundle() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(), "blah".to_string()));

        let mut query = world.query_one::<(&Transform, &String)>(entity).unwrap();
        assert!(query.get().is_some());
    }

    #[test]
    fn query_one_mut_fails_on_nonexisting_bundle() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));

        assert!(world
            .query_one_mut::<(&Transform, &String)>(entity)
            .is_err());
    }

    #[test]
    fn query_one_mut_succeeds_on_existing_bundle() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(), "blah".to_string()));

        assert!(world.query_one_mut::<(&Transform, &String)>(entity).is_ok());
    }

    #[test]
    fn contains_is_accurate() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));

        assert!(world.contains(entity));
        world.despawn(entity).unwrap();
        assert!(!world.contains(entity));
    }

    #[test]
    fn count_is_accurate() {
        let mut world = World::new();
        let expected_count = 100;
        for _ in 0..expected_count {
            world.spawn((Transform::default(),));
        }

        assert_eq!(world.count(), expected_count);
    }

    #[test]
    fn insert_fails_on_nonexisting_entity() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));
        world.despawn(entity).unwrap();

        assert!(world.insert_one(entity, 0).is_err());
    }

    #[test]
    fn insert_overwrites_previous_component() {
        struct TestStruct {
            pub x: usize,
        }
        let expected_value = 100;
        let mut world = World::new();
        let entity = world.spawn((Transform::default(), TestStruct { x: 0 }));

        world
            .insert_one(entity, TestStruct { x: expected_value })
            .unwrap();

        assert_eq!(world.get::<&TestStruct>(entity).unwrap().x, expected_value);
    }

    #[test]
    fn remove_succeeds_on_preexisting_bundle() {
        struct TestStruct {}
        let mut world = World::new();
        let entity = world.spawn((Transform::default(), TestStruct {}));

        assert!(world.remove::<(Transform, TestStruct)>(entity).is_ok());

        // Bundle should no longer exist
        assert!(world.remove::<(Transform, TestStruct)>(entity).is_err());
    }

    #[test]
    fn remove_fails_on_nonexisting_bundle() {
        struct TestStruct {}
        struct NoStruct {}
        let mut world = World::new();
        let entity = world.spawn((Transform::default(), TestStruct {}));

        // This should fail because NoStruct doesn't exist on the entity
        assert!(world
            .remove::<(Transform, TestStruct, NoStruct)>(entity)
            .is_err());
    }

    #[test]
    fn remove_one_succeeds_on_preexisting_component() {
        struct TestStruct {}
        let mut world = World::new();
        let entity = world.spawn((Transform::default(), TestStruct {}));

        assert!(world.remove_one::<TestStruct>(entity).is_ok());
        assert!(world.remove_one::<TestStruct>(entity).is_err());
    }

    #[test]
    fn remove_one_fails_on_nonexisting_component() {
        struct TestStruct {}
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));

        assert!(world.remove_one::<TestStruct>(entity).is_err());
    }

    mod physics_tests {
        use rapier2d::prelude::RigidBodyBuilder;

        use crate::{Transform, World};

        #[test]
        fn add_body_on_preexisting_entity() {
            let mut world = World::new();

            let entity = world.spawn((Transform::default(),));
            let rbh = world
                .physics()
                .build_body(entity, RigidBodyBuilder::dynamic())
                .unwrap();

            assert!(world.physics().rigid_body(rbh).is_some());
        }

        #[test]
        fn remove_body_is_some_on_entity_with_body() {
            let mut world = World::new();

            let (entity, _) = world
                .spawn_with_body((Transform::default(),), RigidBodyBuilder::dynamic())
                .unwrap();
            assert!(world.physics().remove_body(entity).is_some());
        }

        #[test]
        fn remove_body_is_empty_on_entity_with_body() {
            let mut world = World::new();

            let entity = world.spawn((Transform::default(),));
            assert!(world.physics().remove_body(entity).is_none());
        }
    }
}
