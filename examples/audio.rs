use emerald::*;

/// Music found from https://opengameart.org/content/5-chiptunes-action
pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (480, 320);
    settings.render_settings = render_settings;
    emerald::start(
        Box::new(Example {
            elapsed_time: 10.0,
            snd: None,
        }),
        settings,
    )
}

pub struct Example {
    elapsed_time: f32,
    snd: Option<Sound>,
}
impl Game for Example {
    fn initialize(&mut self, mut emd: Emerald) {
        self.snd = Some(
            emd.loader()
                .sound("./examples/assets/test_music.wav")
                .unwrap(),
        );
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();
        self.elapsed_time += emd.delta() as f32;

        if self.elapsed_time >= 10.0 {
            self.elapsed_time = 0.0;

            let snd = self.snd.take().unwrap();
            emd.audio().mixer("test").unwrap().play(snd.clone());

            self.snd = Some(snd);
        }

        let volume = emd.audio().mixer("test").unwrap().get_volume();
        if input.is_key_just_pressed(KeyCode::A) {
            emd.audio().mixer("test").unwrap().set_volume(volume - 0.1);
        } else if input.is_key_just_pressed(KeyCode::D) {
            emd.audio().mixer("test").unwrap().set_volume(volume + 0.1);
        }
    }
}
