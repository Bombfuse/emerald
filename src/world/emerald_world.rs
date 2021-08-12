use crate::rendering::components::Camera;
use crate::EmeraldError;

use hecs::*;

#[cfg(feature = "physics")]
use crate::world::physics::*;
#[cfg(feature = "physics")]
use rapier2d::dynamics::*;

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

    pub fn remove<T: Bundle + 'static>(&mut self, entity: Entity) -> Result<T, ComponentError> {
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
