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
                .pack_bytes(
                    "./examples/assets/Roboto-Light.ttf",
                    include_bytes!("./assets/Roboto-Light.ttf").to_vec(),
                )
                .unwrap();
        }
        let font = emd
            .loader()
            .font("./examples/assets/Roboto-Light.ttf", 40)
            .unwrap();

        let mut left_aligned_label = Label::new("Emerald Engine", font.clone(), 80);
        left_aligned_label.max_width = Some(400.0);

        let mut centered_label = left_aligned_label.clone();
        centered_label.horizontal_align = HorizontalAlign::Center;

        let mut right_label = left_aligned_label.clone();
        right_label.horizontal_align = HorizontalAlign::Right;

        emd.world().spawn((
            ElapsedTime(0.0),
            Position::new(0.0, 0.0),
            left_aligned_label,
        ));
        emd.world().spawn((
            ElapsedTime(0.0),
            Position::new(0.0, 300.0),
            centered_label,
        ));
        emd.world().spawn((
            ElapsedTime(0.0),
            Position::new(0.0, -300.0),
            right_label,
        ));
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();

        for (_, (label, _elapsed_time)) in
            emd.world().query::<(&mut Label, &mut ElapsedTime)>().iter()
        {
            if input.is_key_just_pressed(KeyCode::A) {
                label.scale *= 0.5;
            } else if input.is_key_just_pressed(KeyCode::D) {
                label.scale *= 2.0;
            } else if input.is_key_just_pressed(KeyCode::E) {
                label.max_width = Some(800.0);
            } else if input.is_key_just_pressed(KeyCode::R) {
                label.max_width = Some(400.0);
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
