use crate::*;

pub trait Game {
    fn initialize(&mut self, _emd: Emerald) { }
    fn update(&mut self, _emd: Emerald) { }
    fn draw(&mut self, mut emd: Emerald) {
        let start = Instant::now();
        emd.graphics().begin();
        emd.graphics().draw_world();
        emd.graphics().render();
        let end = Instant::now();

        // TODO(bombfuse): Add draw time to emd.tracer()
        // emd.logger().info(format!("{:?}", end - start));
    }
}