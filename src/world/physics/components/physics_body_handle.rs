use nphysics2d::object::{DefaultBodyHandle, DefaultColliderHandle};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct PhysicsBodyHandle {
    pub(crate) id: usize,
}
impl PhysicsBodyHandle {
    pub(crate) fn new(id: usize) -> Self {
        PhysicsBodyHandle {
            id,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}