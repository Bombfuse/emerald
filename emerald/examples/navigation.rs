use emerald::{
    rendering::components::ColorTri, Emerald, Game, GameSettings, KeyCode, Transform, Vector2,
    World, BLACK, WHITE,
};
use rapier2d::prelude::{ConvexPolygon, Point};

pub fn main() {
    emerald::start(
        Box::new(NavigationExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

pub struct NavigationExample {
    world: World,
}
impl Game for NavigationExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root("./examples/assets/".to_string());

        // TODO: Build navmesh and agents
    }

    fn update(&mut self, emd: Emerald) {
        // TODO: move agents on mouse click
    }

    fn draw(&mut self, mut emd: Emerald<'_>) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
