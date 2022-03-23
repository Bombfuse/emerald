use emerald::{transform::Transform, GameSettings, RenderSettings, EmeraldWorld, Game, Emerald, aseprite_update_system};

pub fn main() {
    let mut settings = GameSettings::default();
    let render_settings = RenderSettings {
        resolution: (320, 180),
        ..Default::default()
    };
    settings.render_settings = render_settings;
    emerald::start(
        MyGame {
            world: EmeraldWorld::new(),
        },
        settings,
    )
}

pub struct MyGame {
    world: EmeraldWorld,
}
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        let mut aseprite = emd
            .loader()
            .aseprite_with_animations("smiley.png", "smiley.json")
            .unwrap();

        aseprite.play_and_loop("smile");

        let mut a2 = aseprite.clone();
        a2.play("smile");

        self.world.spawn((aseprite, Transform::from_translation((64.0, 64.0))));
        self.world.spawn((a2, Transform::from_translation((-64.0, 64.0))));
    }

    fn update(&mut self, emd: Emerald) {
        let delta = emd.delta();

        aseprite_update_system(&mut self.world, delta);
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
