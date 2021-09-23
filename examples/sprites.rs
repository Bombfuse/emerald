use emerald::*;

pub fn main() {
    emerald::start(SpritesExample { world: None }, GameSettings::default())
}

pub struct SpritesExample {
    world: Option<EmeraldWorld>,
}
impl Game for SpritesExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        let sprite = emd.loader().sprite("bunny.png").unwrap();
        self.world = Some(EmeraldWorld::new());

        if let Some(world) = &mut self.world {
            world.spawn((sprite, Position::zero()));
        }
    }

    fn update(&mut self, _emd: Emerald) {}

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();

        if let Some(world) = &mut self.world {
            emd.graphics().draw_world(world).unwrap();
        }

        emd.graphics().render().unwrap();
    }
}
