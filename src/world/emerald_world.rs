use crate::rendering::components::Camera;
use crate::EmeraldError;

use hecs::*;

#[cfg(feature = "physics")]
use rapier2d::dynamics::*;
#[cfg(feature = "physics")]
use crate::world::physics::*;

pub struct EmeraldWorld {
    #[cfg(feature = "physics")]
    pub(crate) physics_engine: PhysicsEngine,
    pub(crate) inner: World,
}
impl EmeraldWorld {
    pub fn new() -> Self {
        EmeraldWorld {
            #[cfg(feature = "physics")]
            physics_engine: PhysicsEngine::new(),
            inner: World::default(),
        }
    }

    pub fn make_active_camera(&mut self, entity: Entity) -> Result<(), EmeraldError> {
        if let Ok(mut next_active_camera) = self.get_mut::<Camera>(entity.clone()) {
            for (_, mut camera) in self.query::<&mut Camera>().iter() {
                camera.is_active = false
            }

            next_active_camera.is_active = true;
        }

        Err(EmeraldError::new(format!(
            "No camera found for entity {:?}",
            entity
        )))
    }

    // TODO(bombfuse): Load an ecs world and physics world into this one.
    pub fn merge(&mut self, _world: EmeraldWorld) -> Result<(), EmeraldError> {
        Ok(())
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
        let rbh = self.physics().build_body(entity.clone(), body_builder)?;

        Ok((entity, rbh))
    }

    pub fn spawn_batch<I>(&mut self, iter: I) -> SpawnBatchIter<'_, I::IntoIter>
    where
        I: IntoIterator,
        I::Item: Bundle,
    {
        self.inner.spawn_batch::<I>(iter)
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        #[cfg(feature = "physics")]
        self.physics_engine.remove_body(entity);

        self.inner.despawn(entity)
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

    pub fn get_mut<T: Component>(&self, entity: Entity) -> Result<RefMut<'_, T>, ComponentError> {
        self.inner.get_mut::<T>(entity)
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Result<Ref<'_, T>, ComponentError> {
        self.inner.get::<T>(entity)
    }

    pub fn reserve_entity(&self) -> Entity {
        self.inner.reserve_entity()
    }

    pub fn insert(
        &mut self,
        entity: Entity,
        components: impl DynamicBundle,
    ) -> Result<(), NoSuchEntity> {
        self.inner.insert(entity, components)
    }

    pub fn remove<T: Bundle>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.inner.remove::<T>(entity)
    }

    pub fn remove_one<T: Component>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.inner.remove_one::<T>(entity)
    }

    #[cfg(feature = "physics")]
    pub fn physics(&mut self) -> PhysicsHandler {
        PhysicsHandler::new(&mut self.physics_engine, &mut self.inner)
    }

    #[cfg(feature = "physics")]
    pub fn physics_ref(&self) -> PhysicsRefHandler {
        PhysicsRefHandler::new(&self.physics_engine)
    }
}
