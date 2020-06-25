use emerald::*;

pub fn main() {
    emerald::start(Box::new(MyGame { }), GameSettings::default())
}

pub struct MyGame;
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        let sprite = emd.loader().sprite("./static/assets/cursor-sheet.png").unwrap();
        let position = Position::new(15.0, 15.0);

        emd.world().insert((), vec![(
            sprite.clone(), position),
            (sprite, Position::new(0.0, 0.0))]);
    }

    fn update(&mut self, emd: Emerald) {
        println!("{}", emd.delta());
    }
}