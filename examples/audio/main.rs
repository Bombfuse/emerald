use emerald::*;


/// Music found from https://opengameart.org/content/5-chiptunes-action
pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.window_size = (480, 320);
    settings.render_settings = render_settings;
    emerald::start(Box::new(Example { elapsed_time: 10.0, snd: None }), settings)
}

pub struct Example { elapsed_time: f32, snd: Option<Sound> }
impl Game for Example {
    fn initialize(&mut self, mut emd: Emerald) {
        self.snd = Some(
            emd.loader()
                .sound("./examples/assets/test_music.wav").unwrap()
        );
    }

    fn update(&mut self, mut emd: Emerald) {
        self.elapsed_time += emd.delta();

        if self.elapsed_time >= 10.0 {
            self.elapsed_time = 0.0;

            let snd = self.snd.take().unwrap();
            emd.audio()
                .play(snd.clone());

            self.snd = Some(snd);
        }
    }
}