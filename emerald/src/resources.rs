use anymap::any::Any;

pub type Resources = anymap::Map<dyn Any + Sync>;
