use emerald::*;

pub fn main() {
    emerald::start(
        Box::new(EntLoadingExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

pub struct EntLoadingExample {
    world: World,
}
impl Game for EntLoadingExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root("./examples/assets/".to_string());
        emd.loader()
            .ent(&mut self.world, Transform::default(), "bunny.ent")
            .unwrap();
    }

    fn update(&mut self, _emd: Emerald) {}

    fn draw(&mut self, mut emd: Emerald<'_>) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
