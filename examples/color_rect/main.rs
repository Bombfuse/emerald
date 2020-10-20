use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (480, 320);
    settings.render_settings = render_settings;
    emerald::start(Box::new(Example {}), settings)
}

pub struct Example;
impl Game for Example {
    fn initialize(&mut self, mut emd: Emerald) {
        let color_rect = ColorRect::default();

        emd.world()
            .inner()
            .spawn((color_rect, Position::new(240.0, 160.0)));
    }
}
