use emerald::{rendering::components::aseprite_update_system, *};

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
        self.world = emd.loader().world("example.wrld").unwrap();
    }

    fn update(&mut self, emd: Emerald) {
        let delta = emd.delta();
        aseprite_update_system(&mut self.world, delta);
        self.world.physics().step(delta);
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
