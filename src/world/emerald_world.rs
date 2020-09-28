use crate::world::physics::*;
use crate::rendering::components::Camera;
use crate::EmeraldError;

use hecs::*;

pub struct EmeraldWorld {
    pub(crate) physics_engine: PhysicsEngine,
    pub(crate) inner: World,
}
impl EmeraldWorld {
    pub fn new() -> Self {
        EmeraldWorld {
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

        Err(EmeraldError::new(format!("No camera found for entity {:?}", entity)))
    }

    pub fn spawn(&mut self, components: impl DynamicBundle) -> Entity {
        self.inner.spawn(components)
    }
    pub fn spawn_batch<I>(&mut self, iter: I) -> SpawnBatchIter<'_, I::IntoIter>
    where
        I: IntoIterator,
        I::Item: Bundle,
    {
        self.inner.spawn_batch::<I>(iter)
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.physics_engine.remove_body(entity);
        self.inner.despawn(entity)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
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

    pub fn physics(&mut self) -> PhysicsHandler {
        PhysicsHandler::new(&mut self.physics_engine, &mut self.inner)
    }

    pub fn physics_ref(&self) -> PhysicsRefHandler {
        PhysicsRefHandler::new(&self.physics_engine)
    }
}