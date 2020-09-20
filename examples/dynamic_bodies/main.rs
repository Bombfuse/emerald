use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (320, 180);
    settings.render_settings = render_settings;
    emerald::start(Box::new(MyGame { }), settings)
}

pub struct MyGame;
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        // Pack all game files into WASM binary
        #[cfg(target_arch = "wasm32")]
        {
            emd.loader().pack_file(
                    "./examples/assets/bunny.png",
                    include_bytes!("../assets/bunny.png").to_vec()
                ).unwrap();
        }

        match emd.loader().sprite("./examples/assets/bunny.png") {
            Ok(sprite) =>  {
                emd.world().inner().spawn((sprite, Position::new(16.0, 16.0)));
            }
            Err(e) => {},
        };
    }
}