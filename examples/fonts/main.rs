use emerald::*;

pub fn main() {
    emerald::start(Box::new(FontsExample { }), GameSettings::default())
}

pub struct FontsExample;
impl Game for FontsExample {
    fn initialize(&mut self, mut emd: Emerald) {
        // Pack all game files into WASM binary
        #[cfg(target_arch = "wasm32")]
        {
            emd.loader()
                .pack_texture(
                    "./static/assets/font.ttf",
                    include_bytes!("./static/assets/font.ttf").to_vec()
                );
        }

        let font = emd.loader().font().unwrap();
        let label = emd.loader().label("Emerald Game Engine", font).unwrap();
        let position = Position::new(15.0, 15.0);

        emd.world().insert((), vec![(
            sprite.clone(), position),
            (label, Position::new(0.0, 0.0))]);
    }
}