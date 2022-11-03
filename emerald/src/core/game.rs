use crate::*;

pub trait Game {
    fn initialize(&mut self, mut _emd: Emerald) {}
    fn update(&mut self, _emd: Emerald) {}
    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().render().unwrap();
    }
}
