use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (320, 180);
    settings.render_settings = render_settings;
    emerald::start(Box::new(MyGame {}), settings)
}

pub struct MyGame;
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        match emd.loader().world("./examples/assets/sample_prefab.json") {
            Ok(world) => {
                emd.pop_world();
                emd.push_world(world);
            }
            Err(e) => {
                println!("{:?}", e);
            }
        };
    }
}
