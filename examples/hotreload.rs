use emerald::*;

pub fn main() {
    emerald::start(
        HotreloadExample {
            world: EmeraldWorld::new(),
        },
        GameSettings::default(),
    )
}

pub struct HotreloadExample {
    world: EmeraldWorld,
}
impl Game for HotreloadExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        let mut sprite = emd.loader().sprite("hotreload_bunny.png").unwrap();
        sprite.scale.x = 5.0;
        sprite.scale.y = 5.0;
        self.world.spawn((sprite, Transform::default()));
    }

    fn update(&mut self, mut emd: Emerald) {
        emd.loader().hotreload();
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
