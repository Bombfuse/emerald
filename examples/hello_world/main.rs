use emerald::*;

pub fn main() {
    emerald::start(Box::new(MyGame { }), GameSettings { })
}

pub struct MyGame;
impl Game for MyGame {
    fn initialize(&mut self, emd: Emerald) {}
}