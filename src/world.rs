#[cfg(feature = "physics")]
pub mod physics;

use crate::rendering::components::Camera;
use crate::EmeraldError;

use hecs::{
    Bundle, Component, DynamicBundle, Entity, NoSuchEntity, Query, QueryBorrow, QueryItem,
    QueryOne, Ref, RefMut, SpawnBatchIter,
};

#[cfg(feature = "physics")]
use crate::world::physics::*;
#[cfg(feature = "physics")]
use rapier2d::dynamics::*;

pub struct World {
    #[cfg(feature = "physics")]
    pub(crate) physics_engine: PhysicsEngine,
    pub(crate) inner: hecs::World,
}
impl World {
    pub fn new() -> Self {
        World {
            #[cfg(feature = "physics")]
            physics_engine: PhysicsEngine::new(),
            inner: hecs::World::default(),
        }
    }

    /// Disable all cameras then set the camera on the given entity as active.
    /// Fails if the given entity does not exist, or does not have a camera.
    #[inline]
    pub fn make_active_camera(&mut self, entity: Entity) -> Result<(), EmeraldError> {
        let mut set_camera = false;
        if let Ok(mut camera) = self.get_mut::<Camera>(entity) {
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

    #[cfg(feature = "physics")]
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
        #[cfg(feature = "physics")]
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

        #[cfg(feature = "physics")]
        {
            self.physics_engine = PhysicsEngine::new();
        }
    }

    pub fn query<Q: Query>(&self) -> QueryBorrow<'_, Q> {
        self.inner.query::<Q>()
    }

    pub fn get_mut<T: Component>(&self, entity: Entity) -> Result<RefMut<'_, T>, EmeraldError> {
        match self.inner.get_mut::<T>(entity.clone()) {
            Ok(component_ref) => Ok(component_ref),
            Err(e) => Err(EmeraldError::new(format!(
                "Error getting component for entity {:?}. {:?}",
                entity, e
            ))),
        }
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Result<Ref<'_, T>, EmeraldError> {
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

    #[cfg(feature = "physics")]
    pub fn physics(&mut self) -> PhysicsHandler<'_> {
        PhysicsHandler::new(&mut self.physics_engine, &mut self.inner)
    }

    #[cfg(feature = "physics")]
    pub fn physics_ref(&self) -> PhysicsRefHandler<'_> {
        PhysicsRefHandler::new(&self.physics_engine)
    }
}
