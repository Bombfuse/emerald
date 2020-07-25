use nphysics2d::object::{DefaultBodyHandle, DefaultColliderHandle};

#[derive(Clone, Debug)]
pub struct PhysicsBodyHandle {
    pub(crate) body_handle: DefaultBodyHandle,
    pub(crate) body_part_count: u8,
    pub(crate) collider_handles: Vec<DefaultColliderHandle>,
}
impl PhysicsBodyHandle {
    pub fn new(body_handle: DefaultBodyHandle) -> Self {
        PhysicsBodyHandle {
            body_handle,
            body_part_count: 0,
            collider_handles: Vec::new(),
        }
    }

    pub fn add_collider(&mut self, collider_handle: DefaultColliderHandle) {
        self.collider_handles.push(collider_handle);
        self.body_part_count += 1;
    }
}