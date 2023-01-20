use emerald::{Emerald, Game, GameSettings};

pub fn main() {
    emerald::start(Box::new(NavigationExample {}), GameSettings::default())
}

pub struct NavigationExample {}
impl Game for NavigationExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root("./examples/assets/".to_string());
    }

    fn update(&mut self, emd: Emerald) {}

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().render().unwrap();
    }
}
