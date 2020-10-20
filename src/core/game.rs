use crate::*;

pub trait Game {
    fn initialize(&mut self, _emd: Emerald) {}
    fn update(&mut self, _emd: Emerald) {}
    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin();
        emd.graphics().draw_world();
        emd.graphics().render();

        // TODO(bombfuse): Add draw time to emd.tracer()
        // emd.logger().info(format!("{:?}", end - start));
    }
}
