use emerald::*;

pub fn main() {
    emerald::start(HotreloadExample { world: None }, GameSettings::default())
}

pub struct HotreloadExample {
    world: Option<EmeraldWorld>,
}
impl Game for HotreloadExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        let mut sprite = emd.loader().sprite("hotreload_bunny.png").unwrap();
        sprite.scale.x = 5.0;
        sprite.scale.y = 5.0;
        self.world = Some(EmeraldWorld::new());

        if let Some(world) = &mut self.world {
            world.spawn((sprite, Position::zero()));
        }
    }

    fn update(&mut self, mut emd: Emerald) {
        emd.loader().hotreload();
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();

        if let Some(world) = &mut self.world {
            emd.graphics().draw_world(world).unwrap();
        }

        emd.graphics().render().unwrap();
    }
}
