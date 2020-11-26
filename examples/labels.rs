use emerald::*;

pub fn main() {
    emerald::start(Box::new(GamepadExample {}), GameSettings::default())
}

pub struct GamepadExample;
impl Game for GamepadExample {
    fn initialize(&mut self, mut emd: Emerald) {
        // Pack all game files into WASM binary
        #[cfg(target_arch = "wasm32")]
        {
            emd.loader()
                .pack_file(
                    "./examples/assets/Roboto-Light.ttf",
                    include_bytes!("../assets/Roboto-Light.ttf").to_vec(),
                )
                .unwrap();
        }

        let font = emd
            .loader()
            .font("./examples/assets/Roboto-Light.ttf", 40)
            .unwrap();

        emd.world().spawn((
            Position::new(0.0, -320.0),
            Label::new("Emerald Engine", font.clone(), 40),
        ));
        emd.world().spawn((
            Position::new(0.0, -160.0),
            Label::new("Emerald Engine", font.clone(), 80),
        ));
        emd.world().spawn((
            Position::new(0.0, 0.0),
            Label::new("Emerald Engine", font.clone(), 120),
        ));
        emd.world().spawn((
            Position::new(0.0, 160.0),
            Label::new("Emerald Engine", font, 160),
        ));
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();

        for (_, label) in emd.world().query::<&mut Label>().iter() {
            if input.is_key_just_pressed(KeyCode::A) {
                label.scale *= 0.5;
            } else if input.is_key_just_pressed(KeyCode::D) {
                label.scale *= 2.0;
            }
        }
    }
}
