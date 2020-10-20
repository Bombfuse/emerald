use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (480, 320);
    settings.render_settings = render_settings;
    emerald::start(Box::new(MyGame {}), settings)
}

pub struct MyGame;
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        // Pack all game files into WASM binary
        #[cfg(target_arch = "wasm32")]
        {
            emd.loader().pack_texture(
                "./examples/assets/bunny.png",
                include_bytes!("../examples/assets/bunny.png").to_vec(),
            );
        }

        let sprite = emd.loader().sprite("./examples/assets/bunny.png").unwrap();

        emd.world().inner().push((sprite, Position::new(0.0, 0.0)));
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut bunny_query = <(&mut Sprite, &Position)>::query();

        for (mut sprite, pos) in bunny_query.iter_mut(emd.world().inner()) {
            if sprite.scale.x < 10.0 {
                sprite.scale.x *= 1.01;
                sprite.scale.y *= 1.01;
            }
        }
    }
}
