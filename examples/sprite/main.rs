use emerald::*;

pub fn main() {
    emerald::start(Box::new(MyGame { }), GameSettings::default())
}

pub struct MyGame;
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        // Pack all game files into WASM binary
        #[cfg(target_arch = "wasm32")]
        {
            emd.loader()
                .pack_texture(
                    "./static/assets/bunny.png",
                    include_bytes!("../static/assets/bunny.png").to_vec()
                );
        }

        let sprite = emd.loader().sprite("./static/assets/bunny.png").unwrap();
        let position = Position::new(15.0, 15.0);

        emd.world().insert((), vec![(
            sprite.clone(), position),
            (sprite, Position::new(0.0, 0.0))]);
    }
}