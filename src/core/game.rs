use crate::*;

pub trait Game {
    fn initialize(&mut self, _emd: Emerald) {}
    fn update(&mut self, _emd: Emerald) {}
    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin();

        if let Some(mut world) = emd.pop_world() {
            emd.graphics().draw_world(&mut world).unwrap();
            emd.push_world(world);
        }

        emd.graphics().render();
    }
}
