use crate::*;

pub struct RigidBodyHandle {

}

pub struct PhysicsEngine {

}
impl PhysicsEngine {
    pub fn new() -> Self {
        PhysicsEngine {
            
        }
    }

    pub(crate) fn step(&mut self, mut world: &mut legion::prelude::World) {
        let non_physics_bodies_query = <(Read<Velocity>, Write<Position>)>::query()
            .filter(!component::<RigidBodyHandle>());
        for (vel, mut pos) in non_physics_bodies_query.iter_mut(world) {
            pos.x += vel.linear.x;
            pos.y += vel.linear.y;
        }
    }
}