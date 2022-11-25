use emerald::*;

pub fn main() {
    emerald::start(
        Box::new(WorldLoadingExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

pub struct WorldLoadingExample {
    world: World,
}
impl Game for WorldLoadingExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root("./examples/assets/".to_string());
        self.world = emd
            .loader()
            .world(
                WorldLoadConfig {
                    transform_offset: Default::default(),
                    custom_component_loader: None,
                },
                "example.wrld",
            )
            .unwrap();
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
