use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (320, 180);
    settings.render_settings = render_settings;
    emerald::start(Box::new(MyGame { }), settings)
}

pub struct MyGame;
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        let mut aseprite = emd.loader()
            .aseprite_with_animations("./examples/assets/smiley.png", "./examples/assets/smiley.json")
            .unwrap();

        aseprite.play_and_loop("smile");

        emd.world().spawn((
            aseprite, 
            Position::new(64.0, 64.0)
        ));
    }
}