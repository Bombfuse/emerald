use nphysics2d::object::{DefaultBodyHandle, DefaultColliderHandle};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct PhysicsBodyHandle {
    pub(crate) body_id: Uuid,
}
impl PhysicsBodyHandle {
    pub(crate) fn new(body_id: Uuid) -> Self {
        PhysicsBodyHandle {
            body_id,
        }
    }
}