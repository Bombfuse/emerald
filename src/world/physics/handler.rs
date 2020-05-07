use crate::world::*;

pub struct PhysicsHandler<'a> {
    world: &'a mut World,
}
impl<'a> PhysicsHandler<'a> {
    pub fn new(world: &'a mut World) -> Self {
        PhysicsHandler {
            world,
        }
    }

    pub fn step(&mut self) {
        self.world.physics_step();
    }
}