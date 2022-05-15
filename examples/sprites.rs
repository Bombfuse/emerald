use emerald::*;

pub fn main() {
    emerald::start(
        Box::new(SpritesExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

pub struct SpritesExample {
    world: World,
}
impl Game for SpritesExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        let sprite = emd.loader().sprite("bunny.png").unwrap();
        self.world.spawn((sprite, Transform::default()));
    }

    fn update(&mut self, _emd: Emerald) {}

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
