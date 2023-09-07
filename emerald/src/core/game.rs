use crate::*;

pub trait Game {
    fn initialize(&mut self, mut _emd: Emerald) {}
    fn update(&mut self, _emd: Emerald) {}

    /// The game has been resumed and will be updated again.
    fn on_resume(&mut self) {}

    /// The game has been suspended and will not be updated until resumed.
    fn on_suspend(&mut self) {}
    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().render().unwrap();
    }
}
