pub mod physics;

pub mod ent;
pub mod world_physics_loader;

use std::collections::HashMap;

use crate::{
    rendering::components::Camera, resources::Resources, AssetLoadConfig, AssetLoader,
    EmeraldError, PhysicsEngine, PhysicsHandler, Transform, WorldMergeHandler,
};

use hecs::{
    Bundle, Component, ComponentRef, DynamicBundle, Entity, EntityRef, NoSuchEntity, Query,
    QueryBorrow, QueryItem, QueryOne, SpawnBatchIter,
};
use rapier2d::prelude::{RigidBodyBuilder, RigidBodyHandle};

use self::{
    ent::{load_ent, EntLoadConfig},
    world_physics_loader::load_world_physics,
};

pub struct World {
    pub(crate) physics_engine: Option<PhysicsEngine>,
    pub(crate) inner: hecs::World,
    resources: Resources,
    merge_handler: Option<WorldMergeHandler>,
}
impl Default for World {
    fn default() -> Self {
        World {
            physics_engine: None,
            inner: hecs::World::default(),
            merge_handler: None,
            resources: Resources::new(),
        }
    }
}
impl World {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_with_merge_handler(world_merge_handler: WorldMergeHandler) -> Self {
        let mut world = World::new();
        world.set_merge_handler(world_merge_handler);
        world
    }

    pub fn set_merge_handler(&mut self, world_merge_handler: WorldMergeHandler) {
        self.merge_handler = Some(world_merge_handler);
    }

    /// Absorbs another world into this one. Resets and changes the Entity ids of the other worlds, when they are merged into this world.
    /// All entities are placed into this world at their current transform.
    /// The camera of the primary world will remain the current camera.
    /// If physics is enabled, will keep its own physics settings.
    /// Returns a map of OldEntity -> NewEntity. If you have components that store Entity references, use this map to update your references.
    pub fn merge(&mut self, mut other_world: World, offset: Transform) -> Result<(), EmeraldError> {
        for (_, transform) in other_world.query::<&mut Transform>().iter() {
            *transform = transform.clone() + offset.clone();
        }

        let mut entity_id_shift_map = HashMap::new();
        let other_entities = other_world
            .inner
            .iter()
            .map(|entity_ref| entity_ref.entity())
            .collect::<Vec<Entity>>();

        for old_id in other_entities {
            let bundle = match other_world.inner.take(old_id.clone()) {
                Err(_) => {
                    return Err(EmeraldError::new(format!(
                        "Entity {:?} does not exist, cannot merge.",
                        old_id
                    )))
                }
                Ok(bundle) => bundle,
            };

            let new_id = self.inner.spawn(bundle);
            entity_id_shift_map.insert(old_id.clone(), new_id.clone());
            self.merge_physics_entity(other_world.physics_engine(), old_id, new_id)?;
        }

        if let Some(merge_handler) = self.merge_handler {
            (merge_handler)(self, &mut other_world, entity_id_shift_map)?;
        }

        Ok(())
    }

    /// Helper function for [`merge`]
    fn merge_physics_entity(
        &mut self,
        other_world_physics: &mut PhysicsEngine,
        old_id: Entity,
        new_id: Entity,
    ) -> Result<(), EmeraldError> {
        // create physics engine if it doesnt exist
        self.physics_engine();

        let mut colliders = Vec::new();
        for c_id in other_world_physics.get_colliders_handles(old_id.clone()) {
            if let Some(collider) = other_world_physics.remove_collider(c_id) {
                colliders.push(collider);
            }
        }

        if let Some(rigid_body) = other_world_physics.remove_body(old_id.clone()) {
            let physics_engine = self.physics_engine.as_mut().unwrap();
            let new_rbh = physics_engine.add_body(new_id.clone(), rigid_body, &mut self.inner)?;

            for collider in colliders {
                physics_engine.add_collider(new_rbh, collider);
            }
        }

        Ok(())
    }

    /// We use this function to get the physics engine because we need to create one if it does not exist yet.
    /// We by default have no engine to save memory and make worlds lighter.
    /// Many worlds do not require physics and by not including it until it's needed,
    /// we're more freely able to spam world creation.
    pub(crate) fn physics_engine(&mut self) -> &mut PhysicsEngine {
        if self.physics_engine.is_none() {
            self.physics_engine = Some(PhysicsEngine::new());
        }

        // I'm not stoked on unwrapping internally, but this should be fine.
        self.physics_engine.as_mut().unwrap()
    }

    #[inline]
    pub fn resources(&mut self) -> &mut Resources {
        &mut self.resources
    }

    #[inline]
    pub fn resources_ref(&self) -> &Resources {
        &self.resources
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
        for (id, camera) in self.query::<&Camera>().iter() {
            if camera.is_active {
                return Some(id);
            }
        }

        None
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
        self.physics_engine().remove_body(entity);

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
        self.physics_engine = None;
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

    /// Collects the ids of all entities with the given components
    pub fn collect<Q: Query>(&self) -> Vec<Entity> {
        self.query::<Q>().iter().map(|(id, _)| id).collect()
    }

    /// Collects the ids of all entities with the given components
    pub fn collect_by<C: Component>(&self) -> Vec<Entity> {
        self.query::<&C>().iter().map(|(id, _)| id).collect()
    }

    pub fn reserve_entity(&self) -> Entity {
        self.inner.reserve_entity()
    }

    /// Whether `entity` still exists
    pub fn contains(&self, entity: Entity) -> bool {
        self.inner.contains(entity)
    }

    /// The number of entities in the world
    pub fn count(&self) -> usize {
        self.inner.len() as usize
    }

    /// Whether or not the entity satisfies the query
    pub fn satisfies<Q: Query>(&self, entity: Entity) -> bool {
        self.inner
            .satisfies::<Q>(entity)
            .ok()
            .map(|b| b)
            .unwrap_or(false)
    }

    /// Whether or not the entity has the component
    pub fn has<C: Component>(&self, entity: Entity) -> bool {
        self.entity(entity)
            .ok()
            .map(|e| e.has::<C>())
            .unwrap_or(false)
    }

    /// Gets a reference to the entity
    pub fn entity(&self, entity: Entity) -> Result<EntityRef<'_>, NoSuchEntity> {
        self.inner.entity(entity)
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
        self.physics_engine();
        PhysicsHandler::new(self.physics_engine.as_mut().unwrap(), &mut self.inner)
    }
}

pub struct WorldLoadConfig {
    pub transform_offset: Transform,
    pub merge_handler: Option<WorldMergeHandler>,
}
impl Default for WorldLoadConfig {
    fn default() -> Self {
        Self {
            transform_offset: Default::default(),
            merge_handler: None,
        }
    }
}

const PHYSICS_SCHEMA_KEY: &str = "physics";
const ENTITIES_SCHEMA_KEY: &str = "entities";

pub(crate) fn load_world(
    loader: &mut AssetLoader<'_>,
    toml: String,
) -> Result<World, EmeraldError> {
    let mut toml = toml.parse::<toml::Value>()?;
    let mut world = World::new();

    if let Some(merge_handler) = loader
        .asset_engine
        .load_config
        .world_load_config
        .merge_handler
    {
        world.set_merge_handler(merge_handler);
    }

    if let Some(table) = toml.as_table_mut() {
        if let Some(physics_val) = table.remove(PHYSICS_SCHEMA_KEY) {
            load_world_physics(loader, &mut world, &physics_val)?;
        }

        if let Some(mut entities_val) = table.remove(ENTITIES_SCHEMA_KEY) {
            if let Some(entities) = entities_val.as_array_mut() {
                for value in entities {
                    // check if this is a ent path reference
                    if let Some(path) = value.get("path") {
                        if let Some(path) = path.as_str() {
                            loader.ent(&mut world, path, Transform::default())?;
                        }
                    } else {
                        load_ent(loader, &mut world, value, Transform::default())?;
                    }
                }
            }
        }

        if let Some(world_resource_loader) = loader.asset_engine.load_config.world_resource_loader {
            for (key, value) in table.to_owned() {
                (world_resource_loader)(loader, &mut world, value, key)?;
            }
        }
    }

    Ok(world)
}

#[cfg(test)]
mod tests {
    use hecs::Entity;

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
    fn collect_test() {
        let mut world = World::new();
        let mut transform_ents = world
            .spawn_batch([(Transform::default(),), (Transform::default(),)])
            .collect::<Vec<Entity>>();
        let str_ents = world
            .spawn_batch([
                (Transform::default(), String::from("test1")),
                (Transform::default(), String::from("test2")),
                (Transform::default(), String::from("test3")),
            ])
            .collect::<Vec<Entity>>();
        transform_ents.extend(str_ents.clone());

        let usize_ent = world.spawn((Transform::default(), 1usize));
        transform_ents.push(usize_ent.clone());

        assert_eq!(world.collect::<&Transform>(), transform_ents);
        assert_eq!(world.collect::<(&Transform, &String)>(), str_ents);
        assert_eq!(world.collect::<(&Transform, &usize)>(), vec![usize_ent]);

        assert_eq!(world.collect_by::<Transform>().len(), 6);
        assert_eq!(world.collect_by::<String>().len(), 3);
        assert_eq!(world.collect_by::<usize>().len(), 1);
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
    fn satisfies_test() {
        let mut world = World::new();

        let id = world.spawn((Transform::default(), 100usize));

        assert!(world.satisfies::<&Transform>(id));
        assert!(world.satisfies::<(&Transform, &usize)>(id));
        assert!(!world.satisfies::<(&Transform, &String)>(id));
        assert!(!world.satisfies::<(&Transform, &usize, &String)>(id));
    }

    #[test]
    fn has_test() {
        let mut world = World::new();
        let id = world.spawn((Transform::default(), 100usize));

        assert!(world.has::<Transform>(id));
        assert!(world.has::<usize>(id));
        assert!(!world.has::<String>(id));
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

    mod resources_tests {
        use crate::World;
        struct MyResource {
            pub value: usize,
        }
        struct MyOtherResource {
            pub other_value: usize,
        }

        #[test]
        fn retains_value() {
            let mut world = World::new();
            let expected_value = 10003;
            world.resources().insert(MyResource {
                value: expected_value,
            });

            assert_eq!(
                world.resources().get::<MyResource>().unwrap().value,
                expected_value
            );
        }

        #[test]
        fn mutates_value() {
            let mut world = World::new();
            let expected_value = 10003;
            let init_value = 0;
            world.resources().insert(MyResource { value: init_value });
            world.resources().get_mut::<MyResource>().unwrap().value = expected_value;

            assert_eq!(
                world.resources().get::<MyResource>().unwrap().value,
                expected_value
            );
        }

        #[test]
        fn holds_different_resources() {
            let mut world = World::new();
            let expected_value = 10003;
            let other_expected_value = 3;
            world.resources().insert(MyResource {
                value: expected_value,
            });
            world.resources().insert(MyOtherResource {
                other_value: other_expected_value,
            });

            assert_eq!(
                world.resources().get::<MyResource>().unwrap().value,
                expected_value
            );

            assert_eq!(
                world
                    .resources()
                    .get::<MyOtherResource>()
                    .unwrap()
                    .other_value,
                other_expected_value
            );
        }
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
