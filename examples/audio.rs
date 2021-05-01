use emerald::*;

/// Music found from https://opengameart.org/content/5-chiptunes-action
pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (480, 320);
    settings.render_settings = render_settings;
    emerald::start(
        Box::new(Example {
        }),
        settings,
    )
}

pub struct Example {
}
impl Game for Example {
    fn initialize(&mut self, mut emd: Emerald) {
        let snd = emd.loader()
                .sound("./examples/assets/test_music.wav")
                .unwrap();
        emd.audio().mixer("test").unwrap().play_and_loop(snd.clone()).unwrap();

    }

    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();

        let volume = emd.audio().mixer("test").unwrap().get_volume();
        if input.is_key_just_pressed(KeyCode::A) {
            emd.audio().mixer("test").unwrap().set_volume(volume - 0.1);
        } else if input.is_key_just_pressed(KeyCode::D) {
            emd.audio().mixer("test").unwrap().set_volume(volume + 0.1);
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin();
        let font = emd.loader().font("./examples/assets/Roboto-Light.ttf", 48).unwrap();
        let volume = emd.audio().mixer("test").unwrap().get_volume();

        let volume_label = Label::new(format!("Volume: {:05.2}", volume), font.clone(), 48);
        let instructions_a = Label::new("A = -0.1", font.clone(), 48);
        let instructions_b = Label::new("D = +0.1", font.clone(), 48);

        emd.graphics().draw_label(&volume_label, &Position::new(240.0, 180.0)).unwrap();
        // emd.graphics().draw_label(&volume_label, &Position::new(240.0, 180.0)).unwrap();
        // emd.graphics().draw_label(&volume_label, &Position::new(240.0, 180.0)).unwrap();

        emd.graphics().render();
    }
}
