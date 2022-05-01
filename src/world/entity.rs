#[derive(Clone, PartialEq, Hash, Debug)]
pub struct Entity {
    pub(crate) id: uuid::Uuid,
    pub(crate) inner: hecs::Entity,
}
impl Entity {
    pub(crate) fn from_hecs(inner: hecs::Entity) -> Self {
        Entity {
            id: uuid::Uuid::new_v4(),
            inner,
        }
    }

    /// Set the inner entity, useful for updating the inner entity when two worlds merge.
    pub(crate) fn set_inner(&mut self, inner: hecs::Entity) {
        self.inner = inner;
    }
}
