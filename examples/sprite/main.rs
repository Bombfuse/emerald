use emerald::*;

pub fn main() {
    emerald::start(Box::new(MyGame { }), GameSettings::default())
}

pub struct MyGame;
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        let sprite = emd.loader().sprite("./examples/assets/Crates.png").unwrap();

        let position = Position::new(100.0, 100.0);

        emd.world().insert((), vec![(sprite.clone(), position), (sprite, Position::new(0.0, 0.0))]);
    }
}