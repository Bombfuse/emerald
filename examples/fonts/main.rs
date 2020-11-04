use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    settings.render_settings.resolution = (480, 320);

    emerald::start(Box::new(FontsExample {}), settings)
}

pub struct FontsExample;
impl Game for FontsExample {
    fn initialize(&mut self, mut emd: Emerald) {
        let font = emd
            .loader()
            .font("./examples/assets/Roboto-Light.ttf")
            .unwrap();
        let mut label = Label::new("@Emerald Game Engine", font, 12);
        let position = Position::new(0.0, 0.0);

        let e = emd.world().spawn((position, label));
    }

    fn update(&mut self, mut emd: Emerald) {
        let delta = emd.delta() as f32;
        let mut input = emd.input();

        for (_, (position, label)) in emd.world().query::<(&mut Position, &mut Label)>().iter() {
            if input.is_key_pressed(KeyCode::Up) {
                position.y += 50.0 * delta;
            } else if input.is_key_pressed(KeyCode::Down) {
                position.y -= 50.0 * delta;
            }

            if input.is_key_just_pressed(KeyCode::J) {
                label.scale *= 0.5;
            } else if input.is_key_just_pressed(KeyCode::L) {
                label.scale *= 2.0;
            }
        }
    }
}
