use crate::input::*;

pub struct<'a> InputHandler<'a> {
    engine: &'a mut InputEngine,
}
impl InputHandler<'a> {
    pub fn new(engine: &'a mut InputEngine) -> Self {
        InputHandler {
            engine,
        }
    }
}