use crate::world::{EmeraldWorld};

pub struct WorldEngine {
    world_stack: Vec<EmeraldWorld>,
}
impl WorldEngine {
    pub(crate) fn new() -> Self {
        WorldEngine {
            world_stack: Vec::new(),
        }
    }

    pub(crate) fn pop(&mut self) -> Option<EmeraldWorld> { self.world_stack.pop() }

    pub(crate) fn push(&mut self, world: EmeraldWorld) { self.world_stack.push(world) }

    // pub(crate) fn world_ref(&self) -> &EmeraldWorld {
    //     if self.world_stack.len() == 0 {
    //         panic!("There are no worlds available to process.");
    //     }

    //     &self.world_stack[self.world_stack.len() - 1]
    // }

    pub(crate) fn world(&mut self) -> &mut EmeraldWorld {
        if self.world_stack.len() == 0 {
            panic!("There are no worlds available to process.");
        }

        let i = self.world_stack.len() - 1;
        self.world_stack.get_mut(i).unwrap()
    }
}