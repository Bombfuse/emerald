use emerald::{tilemap::Tilemap, *};

pub fn main() {
    emerald::start(
        Box::new(TilemapExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

pub struct TilemapExample {
    world: World,
}
impl Game for TilemapExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        let tileset_key = emd.loader().texture("tileset.png").unwrap();
        let mut tilemap = Tilemap::new(tileset_key, Vector2::new(16, 16), 4, 4);

        tilemap.set_tile(0, 0, Some(0)).unwrap();
        tilemap.set_tile(1, 0, Some(1)).unwrap();
        tilemap.set_tile(0, 1, Some(2)).unwrap();
        tilemap.set_tile(1, 1, Some(3)).unwrap();

        self.world.spawn((tilemap, Transform::default()));
    }

    fn update(&mut self, _emd: Emerald) {}

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
