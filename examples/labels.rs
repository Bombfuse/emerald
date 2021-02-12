use emerald::*;

pub fn main() {
    emerald::start(Box::new(GamepadExample {}), GameSettings::default())
}

pub struct ElapsedTime(f32);

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
            ElapsedTime(0.0),
            Position::new(0.0, 0.0),
            Label::new("Emerald Engine", font.clone(), 80),
        ));
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();
        let delta = emd.delta();

        for (_, (label, elapsed_time)) in
            emd.world().query::<(&mut Label, &mut ElapsedTime)>().iter()
        {
            if input.is_key_just_pressed(KeyCode::A) {
                label.scale *= 0.5;
            } else if input.is_key_just_pressed(KeyCode::D) {
                label.scale *= 2.0;
            }

            // elapsed_time.0 = elapsed_time.0 + delta;

            // if elapsed_time.0 >= 0.5 {
            //     elapsed_time.0 = 0.0;
            //     label.visible_characters += 1;

            //     if label.visible_characters > label.text.len() as i64 {
            //         label.visible_characters = 0;
            //     }
            // }
        }
    }
}
