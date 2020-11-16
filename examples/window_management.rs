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
            emd.loader()
                .pack_file(
                    "./examples/assets/bunny.png",
                    include_bytes!("../assets/bunny.png").to_vec(),
                )
                .unwrap();
        }

        match emd.loader().sprite("./examples/assets/bunny.png") {
            Ok(sprite) => {
                emd.world().spawn((sprite, Position::new(16.0, 16.0)));
            }
            Err(_) => {}
        };
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();

        if input.is_key_just_pressed(KeyCode::A) {
            emd.graphics().set_screen_size(960, 640);
        } else if input.is_key_just_pressed(KeyCode::D) {
            emd.graphics().set_screen_size(480, 320);
        }
    }
}
