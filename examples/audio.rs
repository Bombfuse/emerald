use emerald::*;

/// Music found from https://opengameart.org/content/5-chiptunes-action
pub fn main() {
    let mut settings = GameSettings::default();
    let render_settings = RenderSettings {
        resolution: (480, 320),
        ..Default::default()
    };
    settings.render_settings = render_settings;
    emerald::start(Example {}, settings)
}

pub struct Example {}
impl Game for Example {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();

        let volume = emd.audio().mixer("test").unwrap().get_volume().unwrap();
        if input.is_key_just_pressed(KeyCode::A) {
            emd.audio()
                .mixer("test")
                .unwrap()
                .set_volume(volume - 0.1)
                .unwrap();
            emd.audio()
                .mixer("test2")
                .unwrap()
                .set_volume(volume - 0.1)
                .unwrap();
        } else if input.is_key_just_pressed(KeyCode::D) {
            emd.audio()
                .mixer("test")
                .unwrap()
                .set_volume(volume + 0.1)
                .unwrap();
            emd.audio()
                .mixer("test2")
                .unwrap()
                .set_volume(volume + 0.1)
                .unwrap();
        }

        if input.is_key_just_pressed(KeyCode::Space) {
            let snd = emd.loader().sound("test_music.wav").unwrap();
            emd.audio()
                .mixer("test")
                .unwrap()
                .play_and_loop(snd)
                .unwrap();
        }

        if input.is_key_just_pressed(KeyCode::Z) {
            for _ in 0..10 {
                let snd = emd.loader().sound("test_sound.wav").unwrap();
                emd.audio()
                    .mixer("test2")
                    .unwrap()
                    .play(snd.clone())
                    .unwrap();
            }
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        let font = emd.loader().font("Roboto-Light.ttf", 48).unwrap();
        let volume = emd.audio().mixer("test").unwrap().get_volume().unwrap();

        let volume_label = Label::new(format!("Volume: {:05.2}", volume), font, 48);
        emd.graphics()
            .draw_label(&volume_label, &Transform::from_translation((240.0, 180.0)))
            .unwrap();

        emd.graphics().render().unwrap();
    }
}
