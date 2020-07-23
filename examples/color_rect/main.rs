use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.window_size = (480, 320);
    settings.render_settings = render_settings;
    emerald::start(Box::new(Example { }), settings)
}

pub struct Example;
impl Game for Example {
    fn initialize(&mut self, mut emd: Emerald) {
        let color_rect = ColorRect::default();

        emd.world().insert((), vec![(color_rect, Position::new(0.0, 0.0))]);
    }
}