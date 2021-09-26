use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let render_settings = RenderSettings {
        resolution: (320, 180),
        ..Default::default()
    };
    settings.render_settings = render_settings;
    emerald::start(MyGame {}, settings)
}

pub struct MyGame;
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        let mut aseprite = emd
            .loader()
            .aseprite_with_animations("smiley.png", "smiley.json")
            .unwrap();

        aseprite.play_and_loop("smile");

        emd.world().spawn((aseprite, Position::new(64.0, 64.0)));
    }

    fn update(&mut self, mut emd: Emerald) {
        let world = emd.pop_world();
        let delta = emd.delta();

        if let Some(mut world) = world {
            aseprite_update_system(&mut world, delta);
            emd.push_world(world);
        }
    }
}
